// Copyright (C) 2023 Michael Lee <micl2e2@proton.me>
//
// Licensed under the GNU General Public License, Version 3.0 or any later
// version <LICENSE-GPL or https://www.gnu.org/licenses/gpl-3.0.txt>.
//
// This file may not be copied, modified, or distributed except in compliance
// with the license.
//

use clap::builder::StyledStr;
use wda::WdaError;

#[derive(Debug)]
pub enum MafaError {
    Buggy,
    BugFound(u16),
    //
    ClapMatchError(StyledStr),
    //
    InvalidTimeoutPageLoad,
    InvalidTimeoutScript,
    InvalidSocks5Proxy,
    InvalidSourceLang,
    InvalidTargetLang,
    InvalidWords,
    InvalidTwitterUsername,
    InvalidNumTweets,
    InvalidWrapWidth,
    //
    WebDrvCmdRejected(String, String),
    UnexpectedWda(WdaError),
    ///
    /// corresponds to Wda InvalidBrowserProfileId error
    InvalidUseProfile,
    ///
    /// corresponds to Wda BrowserBinaryNotFound error
    FirefoxNotFound,
    CacheRebuildFail(CacheRebuildFailKind),
    CacheNotBuildable,
    AllCachesInvalid,
    DataFetchedNotReachable,
    //
    UpathNotFound,
    UpathLenNotMatched,
    UpathValNotMatched,
    TweetNotRecoginized(u8),
    CamdLevelNotRecoginized(u8),
    CacheCorrupted,
    //
    MafaDataCacheNotFound,
    //
    RequireLogin,
    MustGui,
}

#[derive(Debug)]
pub enum CacheRebuildFailKind {
    UpathNotFound,
    UpathLenNotMatched,
    UpathValNotMatched,
    UpathLenZero,
}

pub type Result<T> = core::result::Result<T, MafaError>;
