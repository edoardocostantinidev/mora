import Config

config :moradb, :http_port, 4000
config :logger, level: :info

config :mnesia,
  dir: '.mnesia/#{Mix.env()}/#{node()}'
