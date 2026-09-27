import copy
from pathlib import Path
import runpy
import unittest

ROOT = Path(__file__).resolve().parents[2]
GUARD = runpy.run_path(str(ROOT / "scripts/check-product-deps"))


def package(name):
    return {"id": name, "name": name, "manifest_path": str(ROOT / GUARD["OWNED"].get(name, "crates/" + name) / "Cargo.toml"),
            "source": None, "dependencies": []}


def dependency(name, **extra):
    return {"name": name, "source": None, "path": str(ROOT / GUARD["OWNED"].get(name, "crates/" + name)), **extra}


class ProductDependencyPolicy(unittest.TestCase):
    def setUp(self):
        self.packages = [package(name) for name in GUARD["ROOTS"]] + [package("research")]
        self.packages[1]["dependencies"] = [dependency(GUARD["ROOTS"][0])]
        self.packages[2]["dependencies"] = [dependency(GUARD["ROOTS"][1])]
        self.packages[3]["dependencies"] = [dependency("reference", source="registry+example", path=None)]
        self.metadata = {"packages": self.packages, "workspace_members": [p["id"] for p in self.packages]}

    def test_owned_closure_excludes_isolated_reference_harness(self):
        self.assertEqual(GUARD["check"](self.metadata), sorted(GUARD["ROOTS"]))

    def test_all_dependency_kinds_targets_and_transitive_paths_are_checked(self):
        cases = [
            {"source": "registry+example", "path": None},
            {"source": "git+example", "path": None},
            {"source": "registry+example", "path": None, "kind": "build"},
            {"source": "registry+example", "path": None, "kind": "dev"},
            {"source": "registry+example", "path": None, "optional": True, "target": "cfg(windows)"},
            {"source": None, "path": str(ROOT / "third_party/vendor")},
            {"source": None, "path": str(ROOT.parent / "outside")},
        ]
        for case in cases:
            with self.subTest(case=case):
                metadata = copy.deepcopy(self.metadata)
                metadata["packages"][0]["dependencies"] = [dependency("external", **case)]
                with self.assertRaises(ValueError):
                    GUARD["check"](metadata)
        metadata = copy.deepcopy(self.metadata)
        metadata["packages"][0]["dependencies"] = [dependency("research")]
        with self.assertRaises(ValueError):
            GUARD["check"](metadata)  # An owned path may not pull in the reference graph.
