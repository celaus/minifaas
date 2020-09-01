use crate::{DataSink, PerformanceIndicators, QuoteRequest, Quotes, BUFFER_SIZE, N_DAYS_SMA};
use chrono::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use xactor::*;



///
/// Actor that saves incoming messages to a ring buffer
///
#[derive(Default, Debug)]
pub struct FunctionCaller {
  pub data_sink: DataSink,
}

impl Service for FunctionCaller {}

#[async_trait::async_trait]
impl Actor for FunctionCaller {
  async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
    ctx.subscribe::<PerformanceIndicators>().await
  }
}

#[async_trait::async_trait]
impl Handler<PerformanceIndicators> for FunctionCaller {
  async fn handle(&mut self, _ctx: &Context<Self>, msg: PerformanceIndicators) {
    let mut buffer = self.data_sink.write().await;
    buffer.push_front(msg);
    buffer.truncate(BUFFER_SIZE);
  }
}


///
/// Actor that downloads stock data for a specified symbol and period
///
pub struct StockDataDownloader;

#[async_trait::async_trait]
impl Handler<QuoteRequest> for StockDataDownloader {
  async fn handle(&mut self, _ctx: &Context<Self>, msg: QuoteRequest) {
    let symbol = msg.symbol.clone();
    // 1h interval works for larger time periods as well (months/years)
    let data = match fetch_ticker_data(
      msg.symbol,
      msg.from,
      msg.to,
      String::from(crate::HISTORIC_INTERVAL),
    )
    .await
    {
      Ok(quotes) => Quotes {
        symbol: symbol.clone(),
        quotes,
      },
      Err(e) => {
        eprintln!("Ignoring API error for symbol '{}': {}", symbol, e);
        Quotes {
          symbol: symbol.clone(),
          quotes: vec![],
        }
      }
    };
    if let Err(e) = Broker::from_registry().await.unwrap().publish(data) {
      eprint!("{}", e);
    }
  }
}

#[async_trait::async_trait]
impl Actor for StockDataDownloader {
  async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
    ctx.subscribe::<QuoteRequest>().await
  }
}

///
/// Actor that saves incoming messages to a ring buffer
///
#[derive(Default, Debug)]
pub struct BufferSink {
  pub data_sink: DataSink,
}

impl Service for BufferSink {}

#[async_trait::async_trait]
impl Actor for BufferSink {
  async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
    ctx.subscribe::<PerformanceIndicators>().await
  }
}

#[async_trait::async_trait]
impl Handler<PerformanceIndicators> for BufferSink {
  async fn handle(&mut self, _ctx: &Context<Self>, msg: PerformanceIndicators) {
    let mut buffer = self.data_sink.write().await;
    buffer.push_front(msg);
    buffer.truncate(BUFFER_SIZE);
  }
}


///
/// Actor for storing incoming messages in a csv file
///
#[derive(Default, Debug)]
pub struct FileSink {
  pub filename: String,
  pub writer: Option<BufWriter<File>>,
}

impl Service for FileSink {}

#[async_trait::async_trait]
impl Actor for FileSink {
  async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
    let mut file = File::create(&self.filename)
      .unwrap_or_else(|_| panic!("Could not open target file '{}'", self.filename));
    let _ = writeln!(
      &mut file,
      "period start,symbol,price,change %,min,max,30d avg"
    );
    self.writer = Some(BufWriter::new(file));
    ctx.subscribe::<PerformanceIndicators>().await
  }

  async fn stopped(&mut self, ctx: &mut Context<Self>) {
    if let Some(writer) = &mut self.writer {
      writer
        .flush()
        .expect("Something happened when flushing. Data loss :(")
    };
    ctx.stop(None);
  }
}

#[async_trait::async_trait]
impl Handler<PerformanceIndicators> for FileSink {
  async fn handle(&mut self, _ctx: &Context<Self>, msg: PerformanceIndicators) {
    if let Some(file) = &mut self.writer {
      let _ = writeln!(
        file,
        "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
        msg.timestamp.to_rfc3339(),
        msg.symbol,
        msg.price,
        msg.pct_change * 100.0,
        msg.period_min,
        msg.period_max,
        msg.last_sma
      );
    }
  }
}

///
/// Actor to create performance indicators from incoming stock data
///
pub struct StockDataProcessor;

#[async_trait::async_trait]
impl Actor for StockDataProcessor {
  async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
    ctx.subscribe::<Quotes>().await
  }
}

#[async_trait::async_trait]
impl Handler<Quotes> for StockDataProcessor {
  async fn handle(&mut self, _ctx: &Context<Self>, mut msg: Quotes) {
    let data = msg.quotes.as_mut_slice();
    if !data.is_empty() {
      // ensure that the data is sorted by time (asc)
      data.sort_by_cached_key(|k| k.timestamp);

      let last_date = Utc.timestamp(data.last().unwrap().timestamp as i64, 0);

      let close_prices: Vec<f64> = data.iter().map(|q| q.close).collect();
      let last_price: f64 = *close_prices.last().unwrap();
      let period_min = min(&close_prices).await.unwrap_or(0.0);
      let period_max = max(&close_prices).await.unwrap_or(0.0);

      let (_, pct_change) = price_diff(&close_prices).await.unwrap_or((0.0, 0.0));
      let sma = n_window_sma(N_DAYS_SMA, &close_prices)
        .await
        .unwrap_or_default();
      let data = PerformanceIndicators {
        timestamp: last_date,
        symbol: msg.symbol,
        price: last_price,
        pct_change,
        period_min,
        period_max,
        last_sma: *sma.last().unwrap_or(&0.0),
      };

      if let Err(e) = Broker::from_registry().await.unwrap().publish(data) {
        eprint!("{}", e);
      }
    }
  }
}
