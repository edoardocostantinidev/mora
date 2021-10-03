import Config

config :moradb, :http_port, 4000
config :logger, level: :debug

config :mnesia,
  dir: '.mnesia/#{Mix.env()}/#{node()}'

import_config "#{Mix.env()}.exs"
