use Mix.Config

config :logger,
  level: :debug

config :libcluster,
  topologies: [
    dev: [
      strategy: Cluster.Strategy.Gossip
    ]
  ]
