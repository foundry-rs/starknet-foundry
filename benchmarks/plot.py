import matplotlib.pyplot as plt
import pandas as pd

BENCHMARKS_FILE = "benchmarks.csv"
THEME = "default"
TOOLS = [
    ("protostar", "Protostar"),
    ("forge", "Starknet Forge"),
    ("cairo_test", "Cairo-Test"),
]


def main():
    benchmarks = pd.read_csv(BENCHMARKS_FILE)

    fig, ax = plt.subplots()
    ax.set_title("Starknet test framework speed comparison")

    test_cases = [
        x + y for x, y in zip(benchmarks["n_unit"], benchmarks["n_integration"])
    ]

    for tool, label in TOOLS:
        ax.plot(test_cases, benchmarks[tool], "--o", label=label)

    ax.set_xlabel("Number of test cases")
    ax.set_ylabel("Time [s]")

    plt.xticks(sorted(test_cases))

    plt.legend()
    plt.show()


if __name__ == "__main__":
    if THEME in plt.style.available:
        plt.style.use(THEME)

    main()
