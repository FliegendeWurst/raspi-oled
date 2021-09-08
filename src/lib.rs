use std::{thread::sleep, time::{self, Duration}};

use gpio_cdev::{EventType, Line, LineRequestFlags};

fn read_events(line: &gpio_cdev::Line, timeout: std::time::Duration) -> Result<Vec<(u64, EventType)>, SensorError> {
    let input = line.request(
        LineRequestFlags::INPUT,
        0,
        "read-data")?;

    let mut last_state = 1;
    let start = time::Instant::now();

    let mut events = Vec::with_capacity(81);
    while start.elapsed() < timeout && events.len() < 81 {
        let new_state = input.get_value()?;
        if new_state != last_state {
            let timestamp = start.elapsed();
            let event_type = if last_state < new_state {
                EventType::RisingEdge
            } else {
                EventType::FallingEdge
            };
            events.push((timestamp.as_micros() as u64, event_type));
            last_state = new_state;
        }
    }
	if events.len() < 81 {
		return Err(SensorError::Timeout);
	}
    Ok(events)
}

fn events_to_data(events: Vec<(u64, EventType)>) -> Vec<u8> {
    events[1..]
        .windows(2)
        .map(|pair| {
            let prev = pair.get(0).unwrap();
            let next = pair.get(1).unwrap();
            match next.1 {
                EventType::FallingEdge => Some(next.0 - prev.0),
                EventType::RisingEdge => None,
            }
        })
        .filter(|&d| d.is_some())
        .map(|elapsed| {
            if elapsed.unwrap() > 35 { 1 } else { 0 }
        }).collect()
}

const MAX_HUMIDITY: u16 = 1000;

fn process_data(mut bits: &[u8]) -> Result<(u16, u16), SensorError> {
    if bits[0] == 1 {
        // definitely incorrect first bit
		// (the humidity can't be this big..)
        bits = &bits[1..];
    }
    let bytes: Vec<u8> = bits
        .chunks(8)
        .map(|chunk| chunk.iter()
            .enumerate()
			// 8 bits, starting with the MSB
            .map(|(bit_idx, &x)| x << (7 - bit_idx))
            .sum()
        ).collect();
    let rh = (bytes[0] as u16) << 8 | bytes[1] as u16;
	if rh > MAX_HUMIDITY {
		return Err(SensorError::HumidityTooHigh);
	}
    let celsius = (bytes[2] as u16) << 8 | bytes[3] as u16;

    if bits.len() >= 40 {
        let cksum: u8 = bits[32..40]
            .iter()
            .enumerate()
            .map(|(idx, &x)| x << (7 - idx))
            .sum();
        let actual_sum = (bytes[0].wrapping_add(bytes[1]).wrapping_add(bytes[2]).wrapping_add(bytes[3])) & 0xff;
        if actual_sum != cksum {
			return Err(SensorError::ChecksumMismatch);
		}
    }
    Ok((rh, celsius))
}

#[test]
fn test_process_data() {
    let x = process_data(&[1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1]).unwrap();
    assert_eq!(471, x.0);
    assert_eq!(268, x.1);
}

#[derive(Debug)]
pub enum SensorError {
	Io(gpio_cdev::Error),
	ChecksumMismatch,
	HumidityTooHigh,
	Timeout
}

impl From<gpio_cdev::Error> for SensorError {
    fn from(error: gpio_cdev::Error) -> Self {
        SensorError::Io(error)
    }
}

pub fn am2302_reading(line: &Line) -> Result<(u16, u16), SensorError> {
	line.request(LineRequestFlags::OUTPUT, 1, "rust-am2302").unwrap();
    sleep(Duration::from_millis(500));
    set_max_priority();
    // set low for 20 ms
    if let Err(e) = line.request(LineRequestFlags::OUTPUT, 0, "rust-am2302") {
		set_normal_priority();
		return Err(SensorError::Io(e));
	}
    sleep(Duration::from_millis(3));

	let events = read_events(&line, Duration::from_secs(1));
	println!("{:?} {:?}", events, events.as_ref().map(|x| x.len()));
	set_normal_priority();
	let events = events?;
	let data = events_to_data(events);
    process_data(&data)
}

fn set_max_priority() {
    unsafe {
        let mut sched_para: libc::sched_param = std::mem::transmute([0u8; std::mem::size_of::<libc::sched_param>()]);
        sched_para.sched_priority = libc::sched_get_priority_max(libc::SCHED_FIFO);
        libc::sched_setscheduler(0, libc::SCHED_FIFO, (&sched_para) as *const libc::sched_param);
    }
}

fn set_normal_priority() {
    unsafe {
        let sched_para: libc::sched_param = std::mem::transmute([0u8; std::mem::size_of::<libc::sched_param>()]);
        libc::sched_setscheduler(0, libc::SCHED_OTHER, (&sched_para) as *const libc::sched_param);
    }
}
