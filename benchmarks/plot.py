import matplotlib.pyplot as plt
import pandas as pd

BENCHMARKS_FILE = "benchmarks.csv"
THEME = "dark_background"
TOOLS = [
    ("protostar", "Protostar"),
    ("forge", "Starknet Forge"),
    ("cairo_test", "Cairo-Test"),
]


def main():
    benchmarks = pd.read_csv(BENCHMARKS_FILE)

    fig, ax = plt.subplots()
    ax.set_title("Starknet test framework speed comparison")

    for tool, label in TOOLS:
        ax.plot(
            benchmarks["n_files"], benchmarks[tool], label=label, linestyle="dashed"
        )

    ax.set_xlabel("Number of files")
    ax.set_ylabel("Time [s]")

    plt.xticks(sorted(list(benchmarks["n_files"])))

    plt.legend()
    plt.show()


if __name__ == "__main__":
    if THEME in plt.style.available:
        plt.style.use(THEME)

    main()
