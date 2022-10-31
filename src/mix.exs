defmodule Mora.MixProject do
  use Mix.Project

  def project do
    [
      app: :mora,
      version: "0.1.1",
      elixir: "~> 1.12",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      elixirc_paths: elixirc_paths(Mix.env())
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {Mora.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:benchee, "~> 1.0", only: :bench},
      {:benchee_html, "~> 1.0", only: :bench},
      {:cowboy, "~> 2.9.0"},
      {:credo, "~> 1.6", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.0", only: [:dev], runtime: false},
      {:libcluster, "~> 3.3.0"},
      {:memento, "~> 0.3.2"},
      {:mix_test_watch, "~> 1.0", only: :dev, runtime: false},
      {:mox, "~> 1.0.1", only: :test},
      {:plug, "~> 1.14.0"},
      {:plug_cowboy, "~> 2.5.2"},
      {:plug_socket, "~> 0.1.0"},
      {:poison, "~> 5.0"},
      {:prioqueue, "~> 0.2.0"},
      {:priority_queue, "~> 1.0"}
    ]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]
end
