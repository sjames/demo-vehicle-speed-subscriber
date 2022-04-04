use core::{num, time};
use std::io::Read;
use std::{sync::Arc, thread};

use async_std::task;
use cyclonedds_rs::dds_topic::DdsEntity;
use cyclonedds_rs::serdes::{Sample, SampleBuffer};
use cyclonedds_rs::*;
use futures::{select, FutureExt};
use vehicle_signals::v2::vehicle::cabin::door::window::Position;
use vehicle_signals::v2::vehicle::Speed;

fn main() {
    // Listener for the participant to listen to the subscription matched status
    let listener = DdsListener::new()
        .on_subscription_matched(|a, b| {
            println!(
                "Subscription matched! {:?} : {:?}",
                unsafe { a.entity() },
                b
            );
        })
        .on_publication_matched(|a, b| {
            println!("Publication matched! {:?} : {:?}", unsafe { a.entity() }, b);
        })
        .hook();

    let participant = ParticipantBuilder::new()
        .with_listener(listener)
        .create()
        .expect("Unable to create participant");
    let subscriber = SubscriberBuilder::new()
        .create(&participant)
        .expect("Unable to create subscriber");
    let mut qos = DdsQos::create().unwrap();
        qos.set_durability(cyclonedds_rs::dds_durability_kind::DDS_DURABILITY_VOLATILE)
        .set_reliability(
            cyclonedds_rs::dds_reliability_kind::DDS_RELIABILITY_BEST_EFFORT,
            std::time::Duration::from_millis(500),
        );

    // Speed topic and reader
    let topic = TopicBuilder::<Speed>::new()
        .create(&participant)
        .expect("Unable to create topic");
    let reader = ReaderBuilder::new()
        .as_async()
        .with_qos(qos.clone())  // Qos can be cloned
        .create(&subscriber, topic)
        .expect("Unable to create reader");

    // Window position topic and reader
    let window_position = TopicBuilder::<Position>::new()
        .create(&participant)
        .expect("Unable to create topic for position");
    let window_position_reader = ReaderBuilder::new()
        .as_async()
        .with_qos(qos)
        .create(&subscriber, window_position)
        .expect("Unable to create reader");

    let mut speed_samples = SampleBuffer::<Speed>::new(1);
    let mut window_position_samples = SampleBuffer::<Position>::new(5);

    task::block_on(async move {
        task::spawn(async move {
            loop {
                if let Ok(num_speed_samples) = reader.take(&mut speed_samples).await {
                    for s in speed_samples.iter() {
                        println!("Received speed: {:?}", s.get().unwrap().value().0)
                    }
                } else {
                    break;
                }
            }
        });

        loop {
            if let Ok(num_window_samples) = window_position_reader
                .take(&mut window_position_samples)
                .await
            {
                for sample in window_position_samples.iter() {
                    println!(
                        "Got {} samples time:{:?}",
                        num_window_samples,
                        std::time::Instant::now()
                    );
                    if let Some(val) = sample.get() {
                        println!(
                            "Got window position : {} {} ",
                            val.value().0 .0,
                            val.value().1
                        );
                    }
                    println!("End samples");
                }
            } else {
                break;
            }
        }
    });
}
