use Mix.Config

config :logger,
  level: :info

config :libcluster,
  topologies: [
    prod: [
      strategy: Cluster.Strategy.Kubernetes.DNS,
      config: [
        mode: :dns,
        service: "mora-svc-headless",
        application_name: "mora"
      ]
    ]
  ]
