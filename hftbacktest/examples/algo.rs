use std::{collections::{HashMap, VecDeque}, fmt::Debug};

use hftbacktest::prelude::*;

pub fn gridtrading<MD, I, R>(
    hbt: &mut I,
    recorder: &mut R,
    relative_half_spread: f64,
    relative_grid_interval: f64,
    grid_num: usize,
    min_grid_step: f64,
    skew: f64,
    order_qty: f64,
    max_position: f64,
) -> Result<(), i64>
where
    MD: MarketDepth,
    I: Bot<MD>,
    <I as Bot<MD>>::Error: Debug,
    R: Recorder,
    <R as Recorder>::Error: Debug,
{
    let tick_size = hbt.depth(0).tick_size() as f64;
    // min_grid_step should be in multiples of tick_size.
    let min_grid_step = (min_grid_step / tick_size).round() * tick_size;
    let mut last_trade_local_ts = 0;
    let mut last_trade_prices = VecDeque::with_capacity(10);
    // Running interval in nanoseconds
    // while ElapseResult::Ok == hbt.elapse(100_000).unwrap() {
    // while ElapseResult::Ok == hbt.wait_next_feed(true,100_000).unwrap() {

    loop {
        let next_feed_resp = hbt.wait_next_feed(true,1_000_000).unwrap();
        if next_feed_resp == ElapseResult::Ok {
            continue;
        }
        if next_feed_resp == ElapseResult::EndOfData {
            break;
        }
        if next_feed_resp == ElapseResult::MarketFeed {
            let last_trades = hbt.last_trades(0);
            if last_trades.len() > 0 {
                // let last_trade = last_trades[last_trades.len() - 1];

                let last_trade_event = last_trades[last_trades.len() - 1].clone();
                let most_recent_trade_local_ts = last_trade_event.local_ts;
                if most_recent_trade_local_ts != last_trade_local_ts {
                    let most_recent_trade_px = last_trade_event.px;
                    let most_recent_trade_qty = last_trade_event.qty;
                    // println!("Last trade: {} {} {}", most_recent_trade_local_ts, most_recent_trade_px, most_recent_trade_qty);
                    if last_trade_prices.len() >= 10 {
                        last_trade_prices.pop_front();
                    }
                    last_trade_prices.push_back(most_recent_trade_px);
                    last_trade_local_ts = most_recent_trade_local_ts;
                    recorder.record(hbt).unwrap();

                    println!("trade_prices: {:?}", last_trade_prices);
                }
            }
        }

        let depth = hbt.depth(0);
        let position = hbt.position(0);

        if depth.best_bid_tick() == INVALID_MIN || depth.best_ask_tick() == INVALID_MAX {
            // Market depth is incomplete.
            continue;
        }

        let mid_price = (depth.best_bid() + depth.best_ask()) as f64 / 2.0;
        let bid = depth.best_bid();
        let ask = depth.best_ask();
        let normalized_position = position / order_qty;

        let relative_bid_depth = relative_half_spread + skew * normalized_position;
        let relative_ask_depth = relative_half_spread - skew * normalized_position;
        let alpha = 0.0;
        let forecast_mid_price = mid_price + alpha;

        let bid_price =
            (forecast_mid_price * (1.0 - relative_bid_depth)).min(depth.best_bid() as f64);
        let ask_price =
            (forecast_mid_price * (1.0 + relative_ask_depth)).max(depth.best_ask() as f64);

        // min_grid_step enforces grid interval changes to be no less than min_grid_step, which
        // stabilizes the grid_interval and keeps the orders on the grid more stable.
        let grid_interval = ((forecast_mid_price * relative_grid_interval / min_grid_step).round()
            * min_grid_step)
            .max(min_grid_step);

        let mut bid_price = (bid_price / grid_interval).floor() * grid_interval;
        let mut ask_price = (ask_price / grid_interval).ceil() * grid_interval;

        //--------------------------------------------------------
        // Updates quotes

        hbt.clear_inactive_orders(Some(0));

        {
            let orders = hbt.orders(0);
            let mut new_bid_orders = HashMap::new();
            if position < max_position && bid_price.is_finite() {
                for _ in 0..grid_num {
                    let bid_price_tick = (bid_price / tick_size).round() as u64;

                    // order price in tick is used as order id.
                    new_bid_orders.insert(bid_price_tick, bid_price);
                    // bid_price -= grid_interval;
                }
            }
            // Cancels if an order is not in the new grid.
            let cancel_order_ids: Vec<u64> = orders
                .values()
                .filter(|order| {
                    order.side == Side::Buy
                        && order.cancellable()
                        && !new_bid_orders.contains_key(&order.order_id)
                })
                .map(|order| order.order_id)
                .collect();
            // Posts an order if it doesn't exist.
            let new_orders: Vec<(u64, f64)> = new_bid_orders
                .into_iter()
                .filter(|(order_id, _)| !orders.contains_key(&order_id))
                .map(|v| v)
                .collect();
            for order_id in cancel_order_ids {
                hbt.cancel(0, order_id, false).unwrap();
            }
            for (order_id, order_price) in new_orders {
                hbt.submit_buy_order(
                    0,
                    order_id,
                    bid,
                    order_qty,
                    TimeInForce::GTX,
                    OrdType::Limit,
                    false,
                )
                .unwrap();
            }
        }

        {
            let orders = hbt.orders(0);
            let mut new_ask_orders = HashMap::new();
            if position > -max_position && ask_price.is_finite() {
                for _ in 0..grid_num {
                    let ask_price_tick = (ask_price / tick_size).round() as u64;

                    // order price in tick is used as order id.
                    new_ask_orders.insert(ask_price_tick, ask_price);

                    ask_price += grid_interval;
                }
            }
            // Cancels if an order is not in the new grid.
            let cancel_order_ids: Vec<u64> = orders
                .values()
                .filter(|order| {
                    order.side == Side::Sell
                        && order.cancellable()
                        && !new_ask_orders.contains_key(&order.order_id)
                })
                .map(|order| order.order_id)
                .collect();
            // Posts an order if it doesn't exist.
            let new_orders: Vec<(u64, f64)> = new_ask_orders
                .into_iter()
                .filter(|(order_id, _)| !orders.contains_key(&order_id))
                .map(|v| v)
                .collect();
            for order_id in cancel_order_ids {
                hbt.cancel(0, order_id, false).unwrap();
            }
            for (order_id, order_price) in new_orders {
                hbt.submit_sell_order(
                    0,
                    order_id,
                    ask,
                    order_qty,
                    TimeInForce::GTX,
                    OrdType::Limit,
                    false,
                )
                .unwrap();
            }
        }
    }
    Ok(())
}
