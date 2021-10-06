use Mix.Config

config :logger,
  level: :info

config :libcluster,
  topologies: [
    prod: [
      strategy: Cluster.Strategy.Kubernetes,
      config: [
        mode: :dns,
        service: "mora-nodes",
        application_name: "mora",
        kubernetes_node_basename: "mora",
        kubernetes_selector: "app=mora",
        kubernetes_namespace: "mora",
        polling_interval: 10_000
      ]
    ]
  ]
