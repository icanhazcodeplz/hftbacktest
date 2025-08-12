use algo::gridtrading;
use hftbacktest::{
    backtest::{
        Backtest,
        ExchangeKind,
        L2AssetBuilder,
        assettype::LinearAsset,
        data::{DataSource, read_npz_file},
        models::{
            CommonFees,
            ConstantLatency,
            PowerProbQueueFunc3,
            ProbQueueModel,
            TradingValueFeeModel,
        },
        recorder::BacktestRecorder,
    },
    prelude::{ApplySnapshot, Bot, HashMapMarketDepth},
};

mod algo;

fn prepare_backtest() -> Backtest<HashMapMarketDepth> {
    // let latency_data = (20240501..20240532)
    //     .map(|date| DataSource::File(format!("latency_{date}.npz")))
    //     .collect();

    let latency_model = ConstantLatency::new(10_000_000,10_000_000);
    let asset_type = LinearAsset::new(1.0);
    let queue_model = ProbQueueModel::new(PowerProbQueueFunc3::new(3.0));
    // 
    // let data = (20240501..20240532)
    //     .map(|date| DataSource::File(format!("btcusdt_{date}.npz")))
    //     .collect();

    let data = Vec::from([DataSource::File(format!("btcusdt_20240809-20000.npz"))]);

    let hbt = Backtest::builder()
        .add_asset(
            L2AssetBuilder::new()
                .data(data)
                .latency_model(latency_model)
                .asset_type(asset_type)
                .fee_model(TradingValueFeeModel::new(CommonFees::new(-0.00005, 0.0007)))
                .exchange(ExchangeKind::NoPartialFillExchange)
                .last_trades_capacity(1000)
                .queue_model(queue_model)
                .depth(|| {
                    let mut depth = HashMapMarketDepth::new(0.01, 0.001);
                    depth.apply_snapshot(
                        &read_npz_file("btcusdt_20240808_eod.npz", "data").unwrap(),
                    );
                    depth
                })
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();
    hbt
}

fn main() {
    tracing_subscriber::fmt::init();

    let relative_half_spread = 0.05;
    let relative_grid_interval = 0.05;
    let grid_num = 2;
    let min_grid_step = 0.01; // tick size
    let skew = relative_half_spread / grid_num as f64;
    let order_qty = 0.1;
    let max_position = grid_num as f64 * order_qty;

    let mut hbt = prepare_backtest();
    let mut recorder = BacktestRecorder::new(&hbt);
    gridtrading(
        &mut hbt,
        &mut recorder,
        relative_half_spread,
        relative_grid_interval,
        grid_num,
        min_grid_step,
        skew,
        order_qty,
        max_position,
    )
    .unwrap();
    hbt.close().unwrap();
    recorder.to_csv("gridtrading", ".").unwrap();
}
