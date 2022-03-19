use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const FUNDAMENTAL_FREQUENCY: f32 = 440.;

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find output device");

    let config = device.default_output_config()?;
    println!("{:?}", &config);

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into())?,
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into())?,
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into())?,
    }

    Ok(())
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let mut curr_frame = 0f32;
    let mut base = 0f32;
    let mut next_value = move |channel: usize| {
        if channel == 1 {
            curr_frame += 1.0;
            let time = curr_frame / sample_rate;
            base = time * FUNDAMENTAL_FREQUENCY * 2.0 * std::f32::consts::PI;
        }
        (channel as f32 * base).sin()
    };

    let err_fn = |err| eprintln!("stream error: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(10000));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut(usize) -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        for (ch, sample) in frame.iter_mut().enumerate() {
            let value: T = cpal::Sample::from::<f32>(&next_sample(ch + 1));
            *sample = value;
        }
    }
}
