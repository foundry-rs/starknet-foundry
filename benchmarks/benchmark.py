from time import perf_counter
import shutil
import pandas as pd
import tempfile
from pathlib import Path
from contextlib import contextmanager
from distutils.dir_util import copy_tree
import subprocess

TOOLCHAINS = [
    # name, command, cairo version
    ("protostar", ["protostar", "test"], 1),
    ("forge", ["forge"], 2),
    ("cairo_test", ["scarb", "cairo-test"], 2),
]

# (unit, integration)
TESTS = [(x, x) for x in range(1, 8)]
CASES_PER_UNIT_TEST = 25
CASES_PER_INTEGRATION_TEST = 15


def log(x):
    print(f"[BENCHMARK] {x}")


@contextmanager
def benchmark_dir(name: str, cairo_version: int, unit: int, integration: int) -> Path:
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        data = Path(__file__).parent / "data"

        copy_tree(str(data / "project"), str(tmp))

        src = tmp / "src"
        tests = tmp / "tests" if name != "cairo_test" else src / "tests"

        for i in range(unit):
            shutil.copy(
                data / "unit_test_template.cairo", tests / f"unit{i}_test.cairo"
            )

        for i in range(integration):
            shutil.copy(
                data / f"{name}.cairo",
                tests / f"integration{i}_test.cairo",
            )

        shutil.copy(data / f"hello_starknet_{cairo_version}.cairo", src / "lib.cairo")

        if name == "cairo_test":
            with open(src / "tests.cairo", "w") as f:
                for i in range(unit):
                    f.write(f"mod unit{i}_test;\n")
                for i in range(integration):
                    f.write(f"mod integration{i}_test;\n")
            with open(src / "lib.cairo", "a") as f:
                f.write("\n")
                f.write("mod tests;\n")

        try:
            log("Creating test directory")
            yield tmp
        finally:
            pass


def benchmark():
    data = {
        "n_files": [],
        "n_unit": [],
        "n_integration": [],
    } | {name: [] for name, _, _ in TOOLCHAINS}
    for unit, integration in TESTS:
        data["n_files"].append(unit + integration)
        data["n_unit"].append(unit * CASES_PER_UNIT_TEST)
        data["n_integration"].append(integration * CASES_PER_INTEGRATION_TEST)
        for name, cmd, ver in TOOLCHAINS:
            with benchmark_dir(name, ver, unit, integration) as project_path:
                log(f"Running {name}")
                start = perf_counter()

                subprocess.run(
                    cmd,
                    stderr=subprocess.DEVNULL,
                    stdout=subprocess.PIPE,
                    check=False,
                    cwd=project_path,
                )

                data[name].append(perf_counter() - start)

    df = pd.DataFrame(data)
    df.to_csv("benchmarks.csv")
    print("", df, "", sep="\n")


if __name__ == "__main__":
    benchmark()
