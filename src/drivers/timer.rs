use core::convert::Infallible;
use crate::traits::wg::timer;
use nb;
use void::Void;

use crate::peripherals::ctimer::{
    Ctimer
};
use crate::time::{
    MicroSeconds
};

pub struct Timer<TIMER>
where
    TIMER: Ctimer,
{
    timer: TIMER,
}


impl <TIMER> Timer<TIMER>
where TIMER: Ctimer {

    pub fn new(timer: TIMER) -> Self{
        Self {
            timer: timer,
        }
    }

    pub fn release(self) -> TIMER {
        self.timer
    }

}

type TimeUnits = MicroSeconds;
impl <TIMER> Timer<TIMER>
where TIMER: Ctimer {
    pub fn lap(&self) -> TimeUnits{
        MicroSeconds(self.timer.tc.read().bits())
    }
}


impl<TIMER> timer::CountDown for Timer<TIMER> 
where TIMER: Ctimer
{
    type Time = TimeUnits;

    fn start<T>(&mut self, count: T) 
    where T: Into<Self::Time>
    {
        // Match should reset and stop timer, and generate interrupt.
        self.timer.mcr.modify(|_,w| {
            w.mr0i().set_bit() 
            .mr0r().set_bit()
            .mr0s().set_bit()
        } );

        // Set match to target time.  Ctimer fixed input 1MHz.
        self.timer.mr[0].write(|w| unsafe { w.bits(count.into().0) });

        // No divsion necessary.
        self.timer.pr.write(|w| unsafe {w.bits(0)});

        // clear interrupt
        self.timer.ir.modify(|_,w| { w.mr0int().set_bit() });

        // Start timer
        self.timer.tcr.write(|w| {
            w.crst().clear_bit()
            .cen().set_bit()
        });
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.timer.ir.read().mr0int().bit_is_set() {
            self.timer.tcr.write(|w| {
                w.crst().set_bit()
                .cen().clear_bit()
            });
            return Ok(());
        }

        Err(nb::Error::WouldBlock)
    }
}

impl<TIMER> timer::Cancel for Timer<TIMER>
where TIMER: Ctimer
{
    type Error = Infallible;
    fn cancel(&mut self) -> Result<(), Self::Error>{
        self.timer.tcr.write(|w| {
            w.crst().set_bit()
            .cen().clear_bit()
        });
        Ok(())
    }
}