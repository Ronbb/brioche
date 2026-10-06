import copy
import io
import json
from pathlib import Path
import struct
import tarfile
import tempfile
import unittest
import wave

import align


def fixture():
    buffer = io.BytesIO()
    with wave.open(buffer, "wb") as w:
        w.setparams((1, 2, 24000, 0, "NONE", "not compressed"))
        w.writeframes(bytes(48000))
    data = buffer.getvalue()
    sha = align.digest(data)
    text = "C’est une baguette."
    request = {"compilerVersion": align.COMPILER,
               "voice": {"characterId": "test-role", "characterRevision": 1, "voiceRevision": 1},
               "profile": {"locale": "fr-FR", "voiceId": "test-voice", "rate": 1.0},
               "parameters": {"model": "qwen-audio-3.1-tts-flash", "input": {"text": text, "voice": "test-voice"}}}
    key = align.digest(align.json_bytes(request, sort=True))
    words = [{"text": "C’est", "start": 0, "end": 5}, {"text": "une", "start": 6, "end": 9},
             {"text": "baguette", "start": 10, "end": 18}]
    target = {"pointer": "/blocks/0/turns/0", "blockId": "dialogue", "entryId": "line",
              "text": text, "voice": request["voice"], "emotion": "Friendly", "generationKey": key,
              "words": [{"segmentId": "part", "text": w["text"], "segmentStart": w["start"],
                         "segmentEnd": w["end"], "entryStart": w["start"], "entryEnd": w["end"]} for w in words]}
    plan = {"compilerVersion": align.COMPILER, "lessonId": "synthetic", "lessonRevision": 1,
            "sourceHash": "b" * 64, "planHash": "", "targets": [target], "requests": {key: request},
            "totalRequestCharacters": len(text)}
    plan["planHash"] = align.plan_hash(plan)
    name = f"media/{sha}.wav"
    clip = {"id": "c" * 32, "generationKey": key, "words": words, "file": name, "providerFile": name,
            "result": {"sha256": sha, "providerSha256": sha, "byteLength": len(data), "durationMs": 1000},
            "review": {"actorId": 1, "reason": "Synthetic test hearing statement"}}
    return {"schemaVersion": "1.0", "planId": "a" * 32, "plan": plan, "clips": [clip]}, {name: data}


def archive(path, manifest, members, extra=None):
    with tarfile.open(path, "w", format=tarfile.GNU_FORMAT) as tar:
        all_members = {"manifest.json": align.json_bytes(manifest), **members}
        for name, data in all_members.items():
            info = tarfile.TarInfo(name); info.size = len(data)
            tar.addfile(info, io.BytesIO(data))
        if extra:
            tar.addfile(*extra)


class ExportTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name) / "export.tar"
        self.manifest, self.members = fixture()

    def check(self):
        archive(self.path, self.manifest, self.members)
        return align.read_export(self.path)

    def test_valid_fixed_export_and_hash_are_retained(self):
        manifest, media, sha = self.check()
        self.assertEqual(manifest, self.manifest)
        self.assertEqual(set(media), {"manifest.json", *self.members})
        self.assertEqual(sha, align.digest(self.path.read_bytes()))

    def test_hashes_match_actual_rust_compiler_vector(self):
        # Public demo source + synthetic Léa profile, never a production voice decision.
        plan = json.loads(Path(__file__).with_name("rust-plan.fixture.json").read_text(encoding="utf-8"))
        self.assertEqual(align.plan_hash(plan), plan["planHash"])
        self.assertEqual(len(plan["requests"]), 19)
        for key, request in plan["requests"].items():
            self.assertEqual(align.digest(align.json_bytes(request, sort=True)), key)

    def test_tampered_plan_request_review_and_word_ranges_fail(self):
        original = copy.deepcopy(self.manifest)
        mutations = [lambda m: m["plan"].update(lessonRevision=2),
                     lambda m: m["clips"][0]["result"].update(durationMs=999),
                     lambda m: m["clips"][0]["review"].update(actorId=False),
                     lambda m: m["clips"][0]["words"][0].update(end=4),
                     lambda m: m["clips"][0]["words"].pop(1),
                     lambda m: m["clips"].append(copy.deepcopy(m["clips"][0]))]
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                self.manifest = copy.deepcopy(original); mutation(self.manifest)
                with self.assertRaises(ValueError): self.check()

    def test_corruption_and_unreferenced_files_fail(self):
        self.members[next(iter(self.members))] = b"corrupt"
        with self.assertRaises(ValueError): self.check()
        self.manifest, self.members = fixture()
        self.members["media/" + "f" * 64 + ".wav"] = b"unused"
        with self.assertRaises(ValueError): self.check()

    def test_path_traversal_symlink_duplicates_and_compression_fail(self):
        for name, kind in [("../outside.wav", tarfile.REGTYPE), ("manifest.json", tarfile.REGTYPE),
                           ("media/" + "f" * 64 + ".wav", tarfile.SYMTYPE)]:
            info = tarfile.TarInfo(name); info.type = kind
            archive(self.path, self.manifest, self.members, (info, io.BytesIO()))
            with self.assertRaises(ValueError): align.read_export(self.path)
        with tarfile.open(self.path, "w:gz"): pass
        with self.assertRaises(tarfile.ReadError): align.read_export(self.path)

    def test_original_must_match_exact_supported_riff_repair(self):
        clip = self.manifest["clips"][0]
        original = bytearray(self.members[clip["file"]])
        struct.pack_into("<I", original, 4, 2147483583)
        struct.pack_into("<I", original, 40, 2147483647)
        sha = align.digest(original); name = f"media/{sha}.wav"
        clip["providerFile"] = name; clip["result"]["providerSha256"] = sha
        self.members[name] = bytes(original)
        self.check()
        original[-1] = 1
        changed_sha = align.digest(original); changed_name = f"media/{changed_sha}.wav"
        self.members.pop(name); self.members[changed_name] = bytes(original)
        clip["providerFile"] = changed_name; clip["result"]["providerSha256"] = changed_sha
        with self.assertRaises(ValueError): self.check()

    def test_json_duplicates_nonfinite_and_depth_fail(self):
        for text in [b'{"a":1,"a":2}', b'{"x":NaN}', ("[" * 70 + "0" + "]" * 70).encode(), br'{"x":"\ud800"}']:
            with self.assertRaises(ValueError): align.parse_json(text)


class TimingTests(unittest.TestCase):
    def test_exact_original_unicode_offsets_and_decimal_adjacency(self):
        words = fixture()[0]["clips"][0]["words"]
        raw = [{"text": "C'est", "startSeconds": 0, "endSeconds": 0.56},
               {"text": "une", "startSeconds": 0.56, "endSeconds": 0.64},
               {"text": "baguette", "startSeconds": 0.64, "endSeconds": 1.0}]
        result, issues = align.predictions(words, raw, 1000)
        self.assertEqual(issues, [])
        self.assertEqual(result[0]["text"], "C’est")
        self.assertEqual(result[0]["endMs"], result[1]["startMs"])
        self.assertEqual(align.model_token("cafe\u0301"), "café")

    def test_bad_predictions_never_get_fabricated_intervals(self):
        word = [{"text": "Bonjour", "start": 0, "end": 7}]
        for raw in [[], [{"text": "Salut", "startSeconds": 0, "endSeconds": 1}],
                    [{"text": "Bonjour", "startSeconds": 0, "endSeconds": 0}],
                    [{"text": "Bonjour", "startSeconds": -1, "endSeconds": 1}],
                    [{"text": "Bonjour", "startSeconds": 0, "endSeconds": 2}],
                    [{"text": "Bonjour", "startSeconds": float("nan"), "endSeconds": 1}]]:
            result, issues = align.predictions(word, raw, 1000)
            self.assertEqual(result, []); self.assertTrue(issues)

    def test_segment_split_requires_explicit_review(self):
        manifest, _ = fixture()
        target = manifest["plan"]["targets"][0]
        target["words"][0].update(text="C", entryEnd=1, segmentEnd=1)
        words = [{**w, "startMs": 0, "endMs": 100} for w in manifest["clips"][0]["words"]]
        self.assertEqual(align.target_timings(target, words), ([], ["segmentWordBoundaryMismatch"]))

    def test_overlapping_predictions_are_not_repaired(self):
        words = [{"text": "un", "start": 0, "end": 2}, {"text": "café", "start": 3, "end": 7}]
        raw = [{"text": "un", "startSeconds": 0, "endSeconds": 0.6},
               {"text": "café", "startSeconds": 0.5, "endSeconds": 1}]
        self.assertEqual(align.predictions(words, raw, 1000), ([], ["invalidTimeRange"]))

    def test_model_snapshot_corruption_is_rejected_before_importing_ml(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / align.MODEL["revision"]
            path.mkdir()
            (path / next(iter(align.MODEL["files"]))).write_bytes(b"corrupt")
            with self.assertRaisesRegex(ValueError, "model snapshot hash mismatch"):
                align.verify_model(path)

    def test_private_output_rejects_public_path_and_never_overwrites(self):
        with self.assertRaises(ValueError): align.write_private(align.ROOT / "docs/public-result.json", {})
        private = align.ROOT / ".local/private"
        private.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=private) as directory:
            path = Path(directory) / "result.json"
            align.write_private(path, {"reviewRequired": True})
            with self.assertRaises(FileExistsError): align.write_private(path, {})
            self.assertEqual(json.loads(path.read_text()), {"reviewRequired": True})


if __name__ == "__main__":
    unittest.main()
