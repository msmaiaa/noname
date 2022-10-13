/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
	env: {
		API_URL: "http://localhost:1337/api",
		WS_URL: "ws://localhost:1337/ws",
	}
}

module.exports = nextConfig
