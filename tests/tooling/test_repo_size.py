"""Check count boundaries and real Git working-tree/baseline scope."""
import importlib.machinery
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
loader = importlib.machinery.SourceFileLoader("repo_size", str(ROOT / "scripts/repo-size"))
spec = importlib.util.spec_from_loader(loader.name, loader)
size = importlib.util.module_from_spec(spec)
loader.exec_module(size)


class RepositorySize(unittest.TestCase):
    def test_physical_lines_and_binary_bytes(self):
        for data, lines, nonblank in [(b"", 0, 0), (b"a", 1, 1), (b"a\n", 1, 1),
                                     (b"a\r\n \r\n// comment\n", 3, 2)]:
            result = size.measure("crates/terminal/src/example.rs", data)
            self.assertEqual((result["lines"], result["nonblank"]), (lines, nonblank))
        result = size.measure("benchmarks/results/test/image.png", b"\x89PNG\0\n")
        self.assertEqual((result["lines"], result["bytes"], result["binary_files"]), (0, 6, 1))

    def test_working_tree_includes_new_files_excludes_ignored_and_symlinks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def git(*args):
                subprocess.run(["git", *args], cwd=root, check=True, capture_output=True)
            git("init", "-q")
            (root / ".gitignore").write_text("ignored/\n")
            (root / "README.md").write_text("before\n")
            (root / "gone.md").write_text("delete me\n")
            git("add", ".")
            git("-c", "user.name=Size test", "-c", "user.email=size@example.invalid", "commit", "-qm", "fixture")
            (root / "README.md").write_text("after\nsecond\n")
            (root / "gone.md").unlink()
            (root / "ignored").mkdir()
            (root / "ignored/build.rs").write_text("not counted\n")
            (root / "outside-link").symlink_to(ROOT / "Cargo.lock")
            (root / "new.py").write_text("print('new')\n")
            previous = size.committed(root)
            current = size.current(root)
            self.assertEqual({r["path"] for r in current["files"]}, {".gitignore", "README.md", "new.py"})
            self.assertEqual(previous["groups"]["docs"]["lines"], 2)
            self.assertEqual(current["groups"]["python"]["lines"], 1)
            self.assertEqual(current["groups"]["windows"]["lines"], 0)
            self.assertEqual(current["total"]["lines"], 4)
            self.assertEqual(sum(g["lines"] for g in current["groups"].values()), 4)


if __name__ == "__main__":
    unittest.main()
