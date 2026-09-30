# Privacy

Last updated: May 2, 2026

## Overview

Lap is designed as a local-first photo manager. Your photo library is processed on your device, and your files remain under your control.

This document explains what data Lap accesses, what data Lap does not collect by default, and when network access may occur.

## What Lap Accesses

Lap may access the following data on your device in order to provide its features:

- Photos, videos, and folders that you choose to add to a library
- File metadata such as filenames, paths, timestamps, size, format, EXIF data, ratings, tags, comments, and rotation state
- Generated local app data such as thumbnails, previews, indexes, search data, embeddings, face clustering data, video indexing data, and other library-related cache or database records

Lap uses this data to support browsing, search, deduplication, tagging, ratings, file type filtering, video support, face clustering, and other library management features.

## Local Processing

Lap is intended to process your library locally on your device.

By default:

- Your photos and videos are not uploaded to a Lap cloud service
- AI search, smart tags, face detection, face clustering, thumbnail generation, RAW previews, and video indexing are processed locally
- Lap does not include advertising trackers
- Lap does not track daily active usage, session counts, or feature usage patterns
- The frontend does not record user behavior, clicks, or navigation events

## Network Access

Lap operates strictly offline for library processing and management. Limited network access may only occur when explicitly triggered:

- Opening external links such as the project website or GitHub repository in your browser
- Fetching map tiles from OpenStreetMap tile servers when viewing a photo's GPS location on the map

## No Telemetry or Usage Statistics

Lap contains no telemetry, analytics, or tracking services. No call-home mechanisms, session tracking, crash reporters, or usage statistics exist in either the backend or frontend.

**Lap does not and will never send** your photos, videos, folder paths, filenames, search queries, tags, ratings, comments, EXIF data, embeddings, face clusters, thumbnails, previews, database contents, or any other library data.

## Data Storage

Lap stores application data locally on your device. This may include:

- App settings
- Local databases for library records, generated thumbnails and previews, search indexes, embeddings, face clustering data, video indexing data, tags, ratings, comments, and related cache data

This local data is used to provide the app's functionality and improve performance on your device.

## Third-Party Services

Lap does not provide or connect to any cloud storage or telemetry service.

## Changes to This Document

This document may be updated as the app evolves. Privacy-related changes should be reflected in the repository and user-facing documentation.

## Contact

For privacy-related questions or concerns, please open an issue in the project repository:

[https://github.com/julyx10/lap](https://github.com/julyx10/lap)
