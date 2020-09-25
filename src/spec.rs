use clap::Clap;
#[derive(Clap, Debug, Default)]
pub struct TrainSpec {
    #[clap(short, long, default_value = "8000")]
    pub vocab_size: usize,
    #[clap(short, long)]
    pub model_prefix: String,
    pub input: String,
    #[clap(short, long)]
    pub keep_extra_whitespaces: bool,
    #[cfg(debug_assertions)]
    #[clap(long)]
    pub slow: bool,
}
