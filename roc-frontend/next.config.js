/** @type {import('next').NextConfig} */
const nextConfig = {
  // Required for Polkadot WASM modules
  webpack: (config) => {
    config.experiments = { ...config.experiments, asyncWebAssembly: true };
    return config;
  },
};

module.exports = nextConfig;
