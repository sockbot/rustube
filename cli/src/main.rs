#![feature(option_result_contains, bool_to_option)]

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Clap;

use args::DownloadArgs;
use args::StreamFilter;
use rustube::{Error, Id, IdBuf, Stream, Video, VideoFetcher};

use crate::args::{Command, FetchArgs};

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    let command: Command = Command::parse();

    match command {
        Command::Download(args) => download(args).await?,
        Command::Fetch(args) => fetch(args).await?,
    }

    Ok(())
}

async fn download(args: DownloadArgs) -> Result<()> {
    args.logging.init_logger();

    let id = args.identifier.id()?;
    let download_path = download_path(args.filename, args.dir, id.as_borrowed());
    let stream = get_stream(id, args.stream_filter).await?;

    stream.download_to(download_path).await?;

    Ok(())
}

async fn fetch(args: FetchArgs) -> Result<()> {
    args.logging.init_logger();

    let id = args.identifier.id()?;
    let video_info = rustube::VideoFetcher::from_id(id)?
        .fetch_info()
        .await?;

    println!("{:?}", video_info);

    Ok(())
}

async fn get_stream(
    id: IdBuf,
    stream_filter: StreamFilter,
) -> Result<Stream> {
    get_streams(id, &stream_filter)
        .await?
        .max_by(|lhs, rhs| stream_filter.max_stream(lhs, rhs))
        .ok_or(Error::NoStreams)
        .context("There are no streams, that match all your criteria")
}

async fn get_streams<'a>(
    id: IdBuf,
    stream_filter: &'a StreamFilter,
) -> Result<impl Iterator<Item=Stream> + 'a> {
    let streams = get_video(id)
        .await?
        .into_streams()
        .into_iter()
        .filter(move |stream| stream_filter.stream_matches(stream));
    Ok(streams)
}

async fn get_video(id: IdBuf) -> Result<Video> {
    VideoFetcher::from_id(id)?
        .fetch()
        .await
        .context("Could not fetch the video information")?
        .descramble()
        .context("Could not descramble the video information")
}

pub fn download_path(filename: Option<PathBuf>, dir: Option<PathBuf>, video_id: Id<'_>) -> PathBuf {
    let filename = filename
        .unwrap_or_else(|| format!("{}.mp4", video_id.as_str()).into());

    let mut path = dir.unwrap_or_else(PathBuf::new);

    path.push(filename);
    path
}