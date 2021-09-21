defmodule Moradb.MixProject do
  use Mix.Project

  def project do
    [
      app: :moradb,
      version: "0.1.0",
      elixir: "~> 1.12",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {Moradb.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:cowboy, "~> 2.9.0"},
      {:memento, "~> 0.3.2"},
      {:mix_test_watch, "~> 1.0", only: :dev, runtime: false},
      {:plug, "~> 1.12.1"},
      {:plug_cowboy, "~> 2.5.2"},
      {:plug_socket, "~> 0.1.0"},
      {:poison, "~> 3.0"},
      {:prioqueue, "~> 0.2.0"},
      {:priority_queue, "~> 1.0"}
    ]
  end
end
