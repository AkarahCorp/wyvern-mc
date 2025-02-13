#[macro_export]
macro_rules! timeout {
    ($fut:expr, $timer:expr) => {
        match async {
            Timer::after($timer).await;
            Result::Err(())
        }
        .race(async { Result::Ok($fut) })
        .await
        {
            Ok(v) => v,
            Err(_) => {
                return;
            }
        }
    };
}
