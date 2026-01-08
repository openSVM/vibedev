#!/usr/bin/env node

/**
 * Postinstall script for vibedev npm package
 * Downloads the appropriate prebuilt binary for the user's platform
 */

const https = require('https');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execFileSync } = require('child_process');

const PACKAGE_VERSION = require('../package.json').version;
const REPO = 'openSVM/vibedev';
const BINARY_NAME = 'vibedev';

// Map Node.js platform/arch to Rust target triples
const PLATFORM_MAP = {
  'darwin-x64': 'x86_64-apple-darwin',
  'darwin-arm64': 'aarch64-apple-darwin',
  'linux-x64': 'x86_64-unknown-linux-gnu',
  'linux-arm64': 'aarch64-unknown-linux-gnu',
  'win32-x64': 'x86_64-pc-windows-msvc',
};

function getPlatformKey() {
  const platform = os.platform();
  const arch = os.arch();
  return `${platform}-${arch}`;
}

function getBinaryExtension() {
  return os.platform() === 'win32' ? '.exe' : '';
}

function getDownloadUrl(target) {
  // GitHub release asset URL pattern
  // Expected asset name: vibedev-{target}.tar.gz (or .zip for Windows)
  const ext = os.platform() === 'win32' ? 'zip' : 'tar.gz';
  return `https://github.com/${REPO}/releases/download/v${PACKAGE_VERSION}/${BINARY_NAME}-${target}.${ext}`;
}

function download(url) {
  return new Promise((resolve, reject) => {
    const request = (url) => {
      https.get(url, { headers: { 'User-Agent': 'vibedev-installer' } }, (response) => {
        // Handle redirects (GitHub releases redirect to S3)
        if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
          request(response.headers.location);
          return;
        }

        if (response.statusCode !== 200) {
          reject(new Error(`Download failed: HTTP ${response.statusCode}`));
          return;
        }

        const chunks = [];
        response.on('data', (chunk) => chunks.push(chunk));
        response.on('end', () => resolve(Buffer.concat(chunks)));
        response.on('error', reject);
      }).on('error', reject);
    };
    request(url);
  });
}

function extractTarGz(buffer, destDir) {
  const tarPath = path.join(destDir, 'temp.tar.gz');
  fs.writeFileSync(tarPath, buffer);

  try {
    // Use execFileSync with separate arguments (safer than shell execution)
    execFileSync('tar', ['-xzf', tarPath, '-C', destDir], { stdio: 'pipe' });
  } finally {
    fs.unlinkSync(tarPath);
  }
}

function extractZip(buffer, destDir) {
  const zipPath = path.join(destDir, 'temp.zip');
  fs.writeFileSync(zipPath, buffer);

  try {
    // Use execFileSync with separate arguments (safer than shell execution)
    execFileSync('powershell', [
      '-Command',
      `Expand-Archive -Path '${zipPath}' -DestinationPath '${destDir}' -Force`
    ], { stdio: 'pipe' });
  } finally {
    fs.unlinkSync(zipPath);
  }
}

async function main() {
  const platformKey = getPlatformKey();
  const target = PLATFORM_MAP[platformKey];

  if (!target) {
    console.error(`Unsupported platform: ${platformKey}`);
    console.error(`Supported platforms: ${Object.keys(PLATFORM_MAP).join(', ')}`);
    console.error('\nYou can build from source using: cargo install vibedev');
    process.exit(1);
  }

  const binDir = path.join(__dirname, '..', 'bin');
  const binaryPath = path.join(binDir, BINARY_NAME + getBinaryExtension());

  // Skip if binary already exists (e.g., local development)
  if (fs.existsSync(binaryPath)) {
    console.log('vibedev binary already exists, skipping download');
    return;
  }

  const downloadUrl = getDownloadUrl(target);
  console.log(`Downloading vibedev for ${platformKey}...`);
  console.log(`  URL: ${downloadUrl}`);

  try {
    const buffer = await download(downloadUrl);
    console.log(`  Downloaded ${(buffer.length / 1024 / 1024).toFixed(2)} MB`);

    // Ensure bin directory exists
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true });
    }

    // Extract based on platform
    if (os.platform() === 'win32') {
      extractZip(buffer, binDir);
    } else {
      extractTarGz(buffer, binDir);
    }

    // Make binary executable (Unix only)
    if (os.platform() !== 'win32') {
      fs.chmodSync(binaryPath, 0o755);
    }

    // Verify binary exists after extraction
    if (!fs.existsSync(binaryPath)) {
      // Binary might be in a subdirectory, try to find it
      const files = fs.readdirSync(binDir);
      const binary = files.find(f => f.startsWith(BINARY_NAME));
      if (binary && binary !== BINARY_NAME + getBinaryExtension()) {
        fs.renameSync(path.join(binDir, binary), binaryPath);
      }
    }

    if (fs.existsSync(binaryPath)) {
      console.log('vibedev installed successfully!');
    } else {
      throw new Error('Binary not found after extraction');
    }
  } catch (error) {
    console.error(`\nFailed to download prebuilt binary: ${error.message}`);
    console.error('\nAlternative installation methods:');
    console.error('  1. Install Rust and build from source: cargo install vibedev');
    console.error('  2. Download manually from: https://github.com/openSVM/vibedev/releases');
    console.error(`  3. Check if your platform (${platformKey}) is supported`);

    // Don't fail the install - let the wrapper handle the missing binary
    // This allows users to manually place the binary
  }
}

main().catch(console.error);
