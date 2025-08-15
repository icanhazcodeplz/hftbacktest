from datetime import datetime

from numba import njit

import numpy as np

@njit
def trade_side(price: np.float64, bid:np.float64, ask:np.float64) -> str:
    round_digits = 4
    if np.round(price,round_digits) == np.round(bid,round_digits):
        return "A"
    elif np.round(price,round_digits) == np.round(ask,round_digits):
        return "B"
    elif np.round(price,round_digits) >= np.round(ask,round_digits) or np.round(price,round_digits) <= np.round(bid,round_digits):
        raise
    return "N"

@njit
def print_bbo(hbt):
    # Iterating until hftbacktest reaches the end of data.
    # Elapses 60-sec every iteration.
    # Time unit is the same as data's timestamp's unit.
    # Timestamp of the sample data is in nanoseconds.
    # while hbt.elapse(60 * 1e9) == 0:
    previous_trade_time = 0
    while True:
        feed_status = hbt.wait_next_feed(True, 1e8)
        market_feed = False
        if feed_status == 0:
            continue
        elif feed_status == 1:
            break
        elif feed_status == 2:
            market_feed = True
            pass
        elif feed_status == 3:
            print("Order response")
            pass
        else:
            raise RuntimeError(feed_status)
        # Gets the market depth for the first asset.
        depth = hbt.depth(0)
        if hbt.current_timestamp > 1753259184761095168:
            pass
            # break
        # Prints the best bid and the best offer.
        if market_feed:
            trades = hbt.last_trades(0)
            if len(trades) > 0:
                local_ts = trades[-1][2]
                price = trades[-1][3]
                qty = trades[-1][4]
                if local_ts != previous_trade_time:
                    pass

                    side = trade_side(price, depth.best_bid, depth.best_ask)
                    # print(f"{local_ts} | {qty} | {depth.best_bid} | {price} | {depth.best_ask} | {side}")

                previous_trade_time = local_ts
            # print(
            #     ', best_bid:', np.round(depth.best_bid, 1),
            #     ', best_ask:', np.round(depth.best_ask, 1)
            # )
            if len(trades) > 5:
                hbt.clear_last_trades(0)
    return True


from hftbacktest import BacktestAsset, HashMapMarketDepthBacktest

asset = (
    BacktestAsset()
        .data(['PAPL_20250723_mbo2.npz'])
        # Sets the initial snapshot (optional).
        # .initial_snapshot('usdm/btcusdt_20240808_eod.npz')
        # Asset type:
        # * Linear
        # * Inverse.
        # 1.0 represents the contract size, which is the value of the asset per quoted price.
        .linear_asset(1.0)
        .constant_order_latency(30_000_000, 30_000_000)
        .l3_fifo_queue_model()
        .no_partial_fill_exchange()
        # 0.02% maker fee and 0.07% taker fee. If the fee is negative, it represents a rebate.
        # For example, -0.00005 represents a 0.005% rebate for the maker order.
        .trading_value_fee_model(0.0002, 0.0007)
        .tick_size(0.01)  # Tick size of this asset: minimum price increasement
        .lot_size(1)  # Lot size of this asset: minimum trading unit.
        # Sets the capacity of the vector that stores trades occurring in the market.
        # If you set the size, you need call `clear_last_trades` to clear the vector.
        # A value of 0 indicates that no market trades are stored. (Default)
        .last_trades_capacity(10000)
)

hbt = HashMapMarketDepthBacktest([asset])
start = datetime.now()
print_bbo(hbt)
print(datetime.now() - start)
_ = hbt.close()