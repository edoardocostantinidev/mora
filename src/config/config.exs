import Config

config :moradb, :http_port, 4000

config :mnesia,
  dir: '.mnesia/#{Mix.env()}/#{node()}'
