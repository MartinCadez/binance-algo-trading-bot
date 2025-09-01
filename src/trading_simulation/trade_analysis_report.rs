use sqlx::PgPool;
use chrono::{DateTime, Utc};
use crate::utils::objects::{Trade};

// point on the realized equity curve (equity after each CLOSED trade)
#[derive(Debug, Clone)]
pub struct EquityPoint {
    pub time: DateTime<Utc>,
    pub equity: f64,
}

#[derive(Debug, Clone, Default)]
pub struct PnlStats {
    pub total_trades: usize,
    pub winners: usize,
    pub losers: usize,
    pub win_rate: f64,      // [0, 1]
    pub gross_pnl: f64,     // sum of realized pnl over CLOSED trades
    pub avg_win: f64,
    pub avg_loss: f64,      // negative number (or 0 if none)
    pub profit_factor: f64, // sum(wins) / abs(sum(losses))
    pub best_trade: f64,
    pub worst_trade: f64,
}

#[derive(Debug, Clone, Default)]
pub struct HoldingTimeStats {
    pub avg_minutes: f64,
    pub median_minutes: f64,
}

#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub symbol: String,
    pub equity_curve: Vec<EquityPoint>,
    pub pnl_stats: PnlStats,
    pub unrealized_pnl: f64,
    pub open_positions: usize,
    pub holding_time: HoldingTimeStats,
}

impl AnalysisReport {
    /// Simple text formatter (for logs / console)
    pub fn format_text(&self) -> String {
        let stats = &self.pnl_stats;
        format!(
            r#"=== Analysis Report ({symbol}) ===
            Total trades: {tot} | Win rate: {wr:.1}%
            Gross PnL: {gpnl:.2} | Profit factor: {pf:.2}
            Best trade: {best:.2} | Worst trade: {worst:.2}
            Open position: {open} | Unrealized PnL: {unpnl:.2}
            Avg holding time: {avg_ht:.1}m | Median holding time: {med_ht:.1}m
            Equity (last): {last_eq:.2}
            "#,
            symbol = self.symbol,
            tot = stats.total_trades,
            wr = stats.win_rate * 100.0,
            gpnl = stats.gross_pnl,
            pf = stats.profit_factor,
            best = stats.best_trade,
            worst = stats.worst_trade,
            open = self.open_positions,
            unpnl = self.unrealized_pnl,
            avg_ht = self.holding_time.avg_minutes,
            med_ht = self.holding_time.median_minutes,
            last_eq = self.equity_curve.last().map(|e| e.equity).unwrap_or(0.0),
        )
    }
}

/// Get CLOSED trades for a symbol (ordered by exit_time then id)
pub async fn get_closed_trades(pool: &PgPool, symbol: &str) -> Result<Vec<Trade>, sqlx::Error> {
    // cast id::BIGINT to satisfy sqlx type checking.
    sqlx::query_as!(
        Trade,
        r#"
        SELECT
            id::BIGINT as "id!: i64",
            symbol,
            entry_price,
            exit_price,
            position_size,
            trade_size,
            pnl,
            entry_time as "entry_time: chrono::DateTime<chrono::Utc>",
            exit_time  as "exit_time:  chrono::DateTime<chrono::Utc>",
            status
        FROM trades
        WHERE symbol = $1 AND status = 'CLOSED'
        ORDER BY exit_time ASC NULLS LAST, id ASC
        "#,
        symbol
    )
    .fetch_all(pool)
    .await
}

// Get OPEN trades for a symbol (ordered by entry_time then id)
pub async fn get_open_trades(pool: &PgPool, symbol: &str) -> Result<Vec<Trade>, sqlx::Error> {
    sqlx::query_as!(
        Trade,
        r#"
        SELECT
            id::BIGINT as "id!: i64",
            symbol,
            entry_price,
            exit_price,
            amount,
            budget_used,
            pnl,
            entry_time as "entry_time: chrono::DateTime<chrono::Utc>",
            exit_time  as "exit_time:  chrono::DateTime<chrono::Utc>",
            status
        FROM trades
        WHERE symbol = $1 AND status = 'OPEN'
        ORDER BY entry_time ASC, id ASC
        "#,
        symbol
    )
    .fetch_all(pool)
    .await
}

// Get latest close for a symbol from prices
pub async fn get_last_price(pool: &PgPool, symbol: &str) -> Result<Option<f64>, sqlx::Error> {
    // prices uses column `coin`, while your in-memory struct uses `symbol`
    sqlx::query_scalar!(
        r#"
        SELECT close
        FROM prices
        WHERE coin = $1
        ORDER BY timestamp DESC, id DESC
        LIMIT 1
        "#,
        symbol
    )
    .fetch_optional(pool)
    .await
}

// Build a realized equity curve by accumulating CLOSED-trade PnL
pub fn build_equity_curve(initial_balance: f64, closed: &[Trade]) -> Vec<EquityPoint> {
    let mut eq = initial_balance;
    let mut curve = Vec::with_capacity(closed.len());
    for tr in closed {
        if let (Some(pnl), Some(time)) = (tr.pnl, tr.exit_time) {
            eq += pnl;
            curve.push(EquityPoint { time, equity: eq });
        }
    }
    curve
}

// Sum unrealized PnL over all OPEN trades at a given last price
pub fn unrealized_pnl(open_trades: &[Trade], last_price: Option<f64>) -> f64 {
    let Some(lp) = last_price else { return 0.0; };
    open_trades.iter()
        .map(|time| (lp - time.entry_price) * time.trade_size)
        .sum()
}

// aggregate win/loss stats over CLOSED trades
pub fn pnl_stats(closed: &[Trade]) -> PnlStats {
    let mut s = PnlStats::default();
    if closed.is_empty() { return s; }

    s.total_trades = closed.len();
    let mut wins = Vec::new();
    let mut losses = Vec::new();

    for time in closed {
        let stats = time.pnl.unwrap_or(0.0);
        if stats >= 0.0 { wins.push(stats); } else { losses.push(stats); }
    }

    s.winners = wins.len();
    s.losers = losses.len();
    s.win_rate = s.winners as f64 / s.total_trades as f64;
    s.gross_pnl = wins.iter().copied().sum::<f64>() + losses.iter().copied().sum::<f64>();
    s.avg_win = if !wins.is_empty() { wins.iter().copied().sum::<f64>() / wins.len() as f64 } else { 0.0 };
    s.avg_loss = if !losses.is_empty() { losses.iter().copied().sum::<f64>() / losses.len() as f64 } else { 0.0 };
    let sum_wins = wins.iter().copied().sum::<f64>();
    let sum_losses_abs = losses.iter().map(|x| x.abs()).sum::<f64>();
    s.profit_factor = if sum_losses_abs > 0.0 { sum_wins / sum_losses_abs } else { f64::INFINITY };
    s.best_trade = closed.iter().filter_map(|time| time.pnl).fold(f64::NEG_INFINITY, f64::max).max(0.0);
    s.worst_trade = closed.iter().filter_map(|time| time.pnl).fold(f64::INFINITY, f64::min).min(0.0);
    s
}

// average/median holding time over CLOSED trades (minutes)
pub fn holding_time_stats(closed: &[Trade]) -> HoldingTimeStats {
    if closed.is_empty() {
        return HoldingTimeStats::default();
    }
    let mut minutes: Vec<f64> = closed.iter()
        .filter_map(|time| Some(((time.exit_time?) - time.entry_time).num_seconds() as f64 / 60.0))
        .collect();
    if minutes.is_empty() { return HoldingTimeStats::default(); }
    minutes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg = minutes.iter().copied().sum::<f64>() / minutes.len() as f64;
    let med = if minutes.len() % 2 == 1 {
        minutes[minutes.len() / 2]
    } else {
        let mid = minutes.len() / 2;
        (minutes[mid - 1] + minutes[mid]) / 2.0
    };
    HoldingTimeStats { avg_minutes: avg, median_minutes: med }
}

pub async fn generate_report(
    pool: &PgPool,
    symbol: &str,
    initial_balance: f64,
) -> Result<AnalysisReport, sqlx::Error> {
    let closed = get_closed_trades(pool, symbol).await?;
    let open = get_open_trades(pool, symbol).await?;
    let last_price = get_last_price(pool, symbol).await?;

    let curve = build_equity_curve(initial_balance, &closed);
    let unrl = unrealized_pnl(&open, last_price);
    let pnl = pnl_stats(&closed);
    let ht = holding_time_stats(&closed);

    Ok(AnalysisReport {
        symbol: symbol.to_string(),
        equity_curve: curve,
        pnl_stats: pnl,
        unrealized_pnl: unrl,
        open_positions: open.len(),
        holding_time: ht,
    })
}
