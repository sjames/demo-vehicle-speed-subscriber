use core::{num, time};
use std::{sync::Arc, thread};

use cyclonedds_rs::dds_topic::DdsEntity;
use cyclonedds_rs::serdes::{Sample, SampleBuffer};
use cyclonedds_rs::{*};
use vehicle_signals::v2::vehicle::Speed;
use vehicle_signals::v2::vehicle::cabin::door::window::Position;

fn main() {
    //println!("Subscribing vehicle speed every 10ms");

    let mut listener = DdsListener::new();
    let listener = listener.on_subscription_matched(|a,b| {
        println!("Subscription matched!");
    }).on_publication_matched(|a,b|{
        println!("Publication matched");
    }).
    hook();


    let participant = DdsParticipant::create(None, None, Some(listener)).unwrap();
    let subscriber = DdsSubscriber::create(&participant, None, None)
                .expect("Unable to create subscriber");

    let topic = Speed::create_topic(&participant, None, None, None).unwrap();
    let mut reader = DdsReader::create(&subscriber, topic, None, None).unwrap();

    let window_position = Position::create_topic(&participant, None, None, None).unwrap();
    let window_position_reader = DdsReader::create(&subscriber, window_position, None, None).unwrap();

    let delay = time::Duration::from_millis(200);
    let mut speed_samples = SampleBuffer::new(1);
    let mut samples = SampleBuffer::new(5);

    

    loop {
        thread::sleep(delay);
       
       
        
        if let Ok(num_read) = window_position_reader.read_now(&mut samples) {
        for sample in samples.iter() {
            //let data = sample.get().unwrap();
            println!("Got {} samples time:{:?}", num_read, std::time::Instant::now());
            if let Some(val) = sample.get() {
                println!("Got window position : {} {} ", val.value().0.0, val.value().1 );
            }
            println!("End samples");
        }
        } else {
            println!("read failed");
        }

        if let Ok(_) = reader.read_now(&mut speed_samples) {
            for s in speed_samples.iter() {
                println!("Received speed: {:?}", s.get().unwrap().value().0)
            }
        }
        

    }

}
