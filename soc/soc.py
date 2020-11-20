#!/usr/bin/env python3

# This variable defines all the external programs that this module
# relies on.  lxbuildenv reads this variable in order to ensure
# the build will finish without exiting due to missing third-party
# programs.
LX_DEPENDENCIES = ["icestorm", "yosys", "nextpnr-ice40"]

# Import lxbuildenv to integrate the deps/ directory
import lxbuildenv

import argparse

from litex.soc.integration.builder import Builder, builder_args, builder_argdict
from litex.soc.integration.soc_core import soc_core_args, soc_core_argdict, SoCCore

from leech import Platform, CRG
from spi_slave import SPIBridge


# BaseSoC ------------------------------------------------------------------------------------------

class BaseSoC(SoCCore):
    # Statically-define the memory map, to prevent it from shifting across various litex versions.
    SoCCore.mem_map = {
        "csr":              0x00000000,
        "sram":             0x00020000,
    }

    def __init__(self, sys_clk_freq, **kwargs):
        platform = Platform()

        kwargs["cpu_type"] = None
        kwargs["with_uart"] = False
        kwargs["with_timer"] = False
        kwargs["with_ctrl"] = True

        # We don't have RAM or ROM
        kwargs["integrated_sram_size"] = 0
        kwargs["integrated_rom_size"] = 0

        kwargs["csr_data_width"] = 32

        SoCCore.__init__(self, platform, sys_clk_freq, **kwargs)

        self.submodules.crg = CRG(platform, sys_clk_freq)

        # SPI to wishbone bridge
        spi_pads = platform.request("spi_slave")
        self.submodules.bridge = SPIBridge(spi_pads)
        self.bus.add_master(name="bridge", master=self.bridge.wishbone)

        # Suppress yosys output
        assert hasattr(self.platform.toolchain, "build_template")
        if self.platform.toolchain.build_template[0].startswith("yosys "):
            self.platform.toolchain.build_template[0] =\
                self.platform.toolchain.build_template[0].replace("yosys ", "yosys -q ")


def main():
    parser = argparse.ArgumentParser(description="LiteX SoC on Leech")
    parser.add_argument("--sys-clk-freq", type=float, default=48e6, help="Select system clock frequency")
    parser.add_argument("--document-only", action="store_true", help="Do not build a soc. Only generate documentation.")
    builder_args(parser)
    soc_core_args(parser)
    args = parser.parse_args()

    # Create the SOC
    soc = BaseSoC(sys_clk_freq=int(args.sys_clk_freq), **soc_core_argdict(args))

    # Configure command line parameter defaults
    # Don't build software: we don't have CPU.
    builder_kwargs = builder_argdict(args)
    builder_kwargs["compile_software"] = False

    if args.document_only:
        builder_kwargs["compile_gateware"] = False
    if builder_kwargs["csr_svd"] is None:
        builder_kwargs["csr_svd"] = "../litex-pac/soc.svd"

    # Create and run the builder
    builder = Builder(soc, **builder_kwargs)
    builder.build()


if __name__ == "__main__":
    main()
