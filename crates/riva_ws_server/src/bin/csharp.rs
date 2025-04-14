use riva_ws_server::events::{
    Command, Event
};
use riva_ws_server::room::room_id::RoomId;
use serde::{Deserialize, Serialize};
use serde_reflection::{Registry, Samples, Tracer, TracerConfig};
use std::io::Write;
use std::fs::{self, File};
use std::path::Path;
use chrono::{DateTime, Utc};


#[derive(Debug, thiserror::Error)]
pub enum CSharpError {
    #[error("Serde reflection error: {0}")]
    SerdeReflectionError(#[from] serde_reflection::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

fn main() -> Result<(), CSharpError> {




    // Initialize tracer with default configuration
    let mut tracer = Tracer::new(TracerConfig::default());
    let samples = Samples::new();

    

    // Trace all the component types first
    tracer.trace_type::<RoomId>(&samples)?;
    // tracer.trace_simple_type::<DateTime<Utc>>()?;
    
    
    // Trace payload types
    tracer.trace_type::<Command>(&samples)?;
    tracer.trace_type::<Event>(&samples)?;


    // tracer.trace_type::<ClientMessage<JoinRoomPayload>>(&samples)?;
    // tracer.trace_type::<ClientMessage<LeaveRoomPayload>>(&samples)?;
    // tracer.trace_type::<ClientMessage<RequestSlideChangePayload>>(&samples)?;
    // tracer.trace_type::<ClientMessage<RoomJoinedPayload>>(&samples)?;
    // tracer.trace_type::<ClientMessage<RoomLeftPayload>>(&samples)?;
    // tracer.trace_type::<ClientMessage<SlideChangedPayload>>(&samples)?;
    
    // Trace message wrapper types - using trace_simple_type instead of trace_type
    // tracer.trace_type::<JoinRoomMessageType>(&samples)?;
    // tracer.trace_type::<LeaveRoomMessageType>(&samples)?;
    // tracer.trace_type::<RequestSlideChangeMessageType>(&samples)?;
    // tracer.trace_type::<SlideChangedMessageType>(&samples)?;
    // tracer.trace_type::<RoomJoinedMessageType>(&samples)?;
    // tracer.trace_type::<RoomLeftMessageType>(&samples)?;




    // tracer.trace_type::<JoinRoomMessage>(&samples)?;
    // tracer.trace_type::<LeaveRoomMessage>(&samples)?;
    
    // // Trace specific message types
    // tracer.trace_simple_type::<JoinRoomMessage>()?;
    // tracer.trace_simple_type::<LeaveRoomMessage>()?;
    // tracer.trace_simple_type::<RequestSlideChangeMessage>()?;
    // tracer.trace_simple_type::<RoomJoinedMessage>()?;
    // tracer.trace_simple_type::<RoomLeftMessage>()?;
    // tracer.trace_simple_type::<SlideChangedMessage>()?;
    
    // // Finally trace the main enum
    // tracer.trace_simple_type::<PresentationRoomMessage>()?;
    
    // Get the registry, with proper error handling
    let registry = tracer.registry()?;
    
    // Configure the code generator
    let config = serde_generate::CodeGeneratorConfig::new("riva_ws_server".to_string())
        .with_encodings(vec![serde_generate::Encoding::Bincode]);
    let generator = serde_generate::csharp::CodeGenerator::new(&config);
    
    // Create output directory if it doesn't exist
    let output_dir = Path::new("generated/csharp");
    fs::create_dir_all(output_dir)?;

    // Generate C# code and write to files
    generator.write_source_files(output_dir.to_path_buf(), &registry)?;
    
    
    println!("C# code generation completed successfully.");
    println!("Output directory: {}", output_dir.display());
    
    Ok(())
}
