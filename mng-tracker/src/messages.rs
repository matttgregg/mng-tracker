use xactor::*;

/// Message to publish output.
#[message]
#[derive(Clone)]
pub struct PublishTick(pub String);

/// Message to publish an error.
#[message]
#[derive(Clone)]
pub struct PublishError(pub String);

/// Message on a clock tick.
#[message]
#[derive(Clone)]
pub struct Tick {}

/// Message to request a resource flush.
#[message]
#[derive(Clone)]
pub struct Flush {}

/// Message to request termination.
#[message]
#[derive(Clone)]
pub struct Exit {}

/// Message to get last n values.
#[message(result = "Vec<String>")]
#[derive(Clone)]
pub struct LastN(pub usize);
