use iced_aksel::PlotPoint;

use crate::items::Candle;

/// Calculates the Simple Moving Average (SMA) for the given data.
pub fn calculate_sma(data: &[(i64, Candle)], period: usize) -> Vec<PlotPoint> {
    if period == 0 || data.len() < period {
        return Vec::new();
    }

    data.windows(period)
        .map(|window| {
            let sum: f64 = window.iter().map(|(_, candle)| candle.close).sum();
            let avg = sum / period as f64;
            let (timestamp, _) = window.last().unwrap(); // SMA point aligns with the end of the window
            PlotPoint::new(*timestamp as f64, avg)
        })
        .collect()
}

/// Calculates the Bollinger Bands (Upper, Middle, Lower) for the given data.
pub fn calculate_bbands(
    data: &[(i64, Candle)],
    period: usize,
    std_dev_mult: f64,
) -> (Vec<PlotPoint>, Vec<PlotPoint>, Vec<PlotPoint>) {
    if period == 0 || data.len() < period {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let mut upper = Vec::with_capacity(data.len() - period + 1);
    let mut middle = Vec::with_capacity(data.len() - period + 1);
    let mut lower = Vec::with_capacity(data.len() - period + 1);

    for window in data.windows(period) {
        let (timestamp, _) = window.last().unwrap();
        let closes: Vec<f64> = window.iter().map(|(_, c)| c.close).collect();

        // Calculate Mean (which is the SMA)
        let sum: f64 = closes.iter().sum();
        let mean = sum / period as f64;

        // Calculate Standard Deviation
        let variance = closes
            .iter()
            .map(|&close| {
                let diff = close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        let std_dev = variance.sqrt();

        let upper_band = std_dev.mul_add(std_dev_mult, mean);
        let lower_band = std_dev.mul_add(-std_dev_mult, mean);

        upper.push(PlotPoint::new(*timestamp as f64, upper_band));
        middle.push(PlotPoint::new(*timestamp as f64, mean));
        lower.push(PlotPoint::new(*timestamp as f64, lower_band));
    }

    (upper, middle, lower)
}
