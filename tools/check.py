#! /usr/bin/env python3

import json
import subprocess
import sys


def run_inner(args):
    print("Running `{}`...".format(" ".join(args)))
    ret = subprocess.call(args) == 0
    print("")
    return ret


def run(mcu):
    if mcu == "":
        return run_inner(["cargo", "check"])
    else:
        return run_inner(["cargo",
                          "check",
                          "--examples",
                          "--features={}".format(mcu)])


def main():
    cargo_meta = json.loads(
        subprocess.check_output("cargo metadata --no-deps --format-version=1",
                       shell=True,
                       universal_newlines=True)
        )

    crate_info = cargo_meta["packages"][0]

    features = [""] + ["{} rt".format(x)
                       for x in crate_info["features"].keys()
                       if x != "device-selected" and x != "rt"]

    if not all(map(run, features)):
        sys.exit(-1)

if __name__ == "__main__":
    main()

