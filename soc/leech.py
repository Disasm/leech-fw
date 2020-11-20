from migen import *
from migen.genlib.resetsync import AsyncResetSynchronizer
from litex.build.generic_platform import *
from litex.build.lattice import LatticePlatform
from litex.soc.cores.clock import iCE40PLL


# IOs ----------------------------------------------------------------------------------------------

_io = [
    ("spi_slave", 0,
        Subsignal("cs_n", Pins("16"), IOStandard("LVCMOS33")),
        Subsignal("clk",  Pins("15"), IOStandard("LVCMOS33")),
        Subsignal("mosi", Pins("17"), IOStandard("LVCMOS33")),
        Subsignal("miso", Pins("14"), IOStandard("LVCMOS33")),
    ),

    ("clk16", 0, Pins("20"), IOStandard("LVCMOS33")),

    ("irq", 0, Pins("18"), IOStandard("LVCMOS33")),
]


# Platform -----------------------------------------------------------------------------------------

class Platform(LatticePlatform):
    def __init__(self):
        LatticePlatform.__init__(self, "ice40-up5k-sg48", _io, toolchain="icestorm")


# CRG ----------------------------------------------------------------------------------------------

class CRG(Module):
    def __init__(self, platform, sys_clk_freq):
        self.clock_domains.cd_sys = ClockDomain()
        self.clock_domains.cd_por = ClockDomain()

        # Clocks
        clk16 = platform.request("clk16")
        platform.add_period_constraint(clk16, 1e9 / 16e6)
        if sys_clk_freq == 16e6:
            self.comb += self.cd_sys.clk.eq(clk16)
        else:
            self.submodules.pll = pll = iCE40PLL()
            pll.register_clkin(clk16, 16e6)
            pll.create_clkout(self.cd_sys, sys_clk_freq, with_reset=False)
        platform.add_period_constraint(self.cd_sys.clk, 1e9 / sys_clk_freq)

        # Power On Reset
        self.reset = Signal()
        por_cycles = 8
        por_counter = Signal(log2_int(por_cycles), reset=por_cycles - 1)
        self.comb += self.cd_por.clk.eq(self.cd_sys.clk)
        platform.add_period_constraint(self.cd_por.clk, 1e9 / sys_clk_freq)
        self.sync.por += If(por_counter != 0, por_counter.eq(por_counter - 1))
        self.comb += self.cd_sys.rst.eq(por_counter != 0)
        self.specials += AsyncResetSynchronizer(self.cd_por, self.reset)
