use Mix.Config

config :logger,
  level: :error

config :libcluster,
  topologies: [
    test: [
      strategy: Cluster.Strategy.Gossip
    ]
  ]
