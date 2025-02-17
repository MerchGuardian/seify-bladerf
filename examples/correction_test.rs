use anyhow::{Context, Result};
use bladerf::{
    BladeRF, BladeRfAny, Channel, CorrectionDcOffsetI, CorrectionDcOffsetQ, CorrectionGain,
    CorrectionPhase, CorrectionValue,
};

fn main() -> Result<()> {

    // Open the first available BladeRF device
    let device = BladeRfAny::open_first().context("Unable to open BladeRF device")?;

    // Use TX channel 0
    let channel = Channel::Tx1;

    println!("v,I correction,Q correction,phase correction,gain correction");
    for v in (-512..512).step_by(16) {
        // Apply corrections
        device.set_correction(channel, CorrectionDcOffsetI(v))?;
        let corr_i: CorrectionDcOffsetI = device.get_correction(channel)?;

        device.set_correction(channel, CorrectionDcOffsetQ(v))?;
        let corr_q: CorrectionDcOffsetQ = device.get_correction(channel)?;

        device.set_correction(channel, CorrectionPhase(v))?;
        let corr_phase: CorrectionPhase = device.get_correction(channel)?;

        device.set_correction(channel, CorrectionGain(v))?;
        let corr_gain: CorrectionGain = device.get_correction(channel)?;

        println!(
            "{v},{},{},{},{}",
            corr_i.value(),
            corr_q.value(),
            corr_phase.value(),
            corr_gain.value()
        );
    }

    Ok(())
}
