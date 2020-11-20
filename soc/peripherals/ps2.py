from operator import xor

from migen import *
from migen.genlib.cdc import MultiReg
from migen.genlib.misc import WaitTimer
from litex.soc.interconnect.csr_eventmanager import EventManager, EventSourceLevel
from litex.soc.interconnect.stream import SyncFIFO, EndpointDescription
from litex.soc.interconnect.csr import AutoCSR, CSRField, CSRStatus


class PS2Controller(Module, AutoCSR):
    def __init__(self, dat, clk, sys_clk_freq):
        if type(dat) != TSTriple:
            t = TSTriple()
            self.specials += t.get_tristate(dat)
            dat = t

        if type(clk) != TSTriple:
            t = TSTriple()
            self.specials += t.get_tristate(clk)
            clk = t

        dat_i = Signal()
        clk_i = Signal()
        self.specials += [
            MultiReg(dat.i, dat_i),
            MultiReg(clk.i, clk_i),
        ]
        self.comb += [
            dat.oe.eq(0),
            clk.oe.eq(0),
        ]

        fields = [
            CSRField(name="data", size=8, description="Received data"),
            CSRField(name="valid", description="Data valid"),
        ]
        self.rx = CSRStatus(32, description="Receive register", fields=fields)

        self.submodules.ev = EventManager()
        self.ev.data = EventSourceLevel()
        self.ev.finalize()

        layout = EndpointDescription([("data", 8)])
        self.submodules.rx_fifo = rx_fifo = SyncFIFO(layout, 16)
        self.comb += [
            self.rx.fields.data.eq(rx_fifo.source.data),
            self.rx.fields.valid.eq(rx_fifo.source.valid),
            rx_fifo.source.ready.eq(self.rx.we),
            self.ev.data.trigger.eq(rx_fifo.source.valid),
        ]

        # One transaction lasts about 1.1ms, so we can consider the bus idle
        # after 1.5ms of high CLK state
        idle_ticks = int(sys_clk_freq * 1.5e-3)
        reset = Signal()
        self.submodules.reset_timer = WaitTimer(idle_ticks)
        self.comb += [
            self.reset_timer.wait.eq(clk_i),
            reset.eq(self.reset_timer.done),
        ]

        # Stabilize CLK
        clk_stable = Signal()
        prev_clk = Signal()
        self.sync += prev_clk.eq(clk_i)
        self.submodules.clk_timer = WaitTimer(10)
        self.comb += [
            self.clk_timer.wait.eq(prev_clk == clk_i),
            clk_stable.eq(self.clk_timer.done),
        ]
        clk_pulse = Signal()
        clk_stable_d = Signal()
        self.sync += [
            clk_stable_d.eq(clk_stable),
            clk_pulse.eq(clk_stable & ~clk_stable_d)
        ]

        bits = Signal(11)
        bitcount = Signal(4)
        parity = reduce(xor, bits)

        self.sync += [
            rx_fifo.sink.valid.eq(0),
            If(reset,
                bitcount.eq(0),
                bits.eq(0),
            ).Else(
                If(clk_pulse & ~clk_i,
                    bits.eq(Cat(bits[1:], dat_i)),
                    bitcount.eq(bitcount + 1)
                ).Elif(bitcount == 11,
                    If(bits[10] & ~bits[0] & ~parity,
                        rx_fifo.sink.data.eq(bits[1:9]),
                        rx_fifo.sink.valid.eq(1),
                    ),
                    bitcount.eq(0),
                    bits.eq(0)
                )
            )
        ]
