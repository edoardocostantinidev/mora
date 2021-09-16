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
      {:poison, "~> 3.0"},
      {:plug, "~> 1.12.1"},
      {:plug_cowboy, "~> 2.5.2"},
      {:cowboy, "~> 2.9.0"},
      {:priority_queue, "~> 1.0"},
      {:plug_socket, "~> 0.1.0"}
    ]
  end
end
