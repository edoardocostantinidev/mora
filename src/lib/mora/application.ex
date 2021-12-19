defmodule Mora.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application
  require Logger
  @impl true
  def start(_type, _args) do
    port = System.get_env("PORT", "4000")
    Logger.info("Starting Mora DB on port #{inspect(port)}")

    children = [
      {Cluster.Supervisor, [Application.fetch_env!(:libcluster, :topologies)]},
      %{
        id: :pg,
        start: {:pg, :start_link, []}
      },
      {
        Plug.Cowboy,
        scheme: :http,
        plug: Mora,
        options: [
          port: String.to_integer(port),
          dispatch: PlugSocket.plug_cowboy_dispatch(Mora.Api)
        ]
      },
      {Mora.TemporalQueue.DynamicSupervisor,
       strategy: :one_for_one, name: Mora.TemporalQueue.DynamicSupervisor},
      Mora.Database.Mnesia,
      Mora.Dispatchers.Websocket,
      {Registry, keys: :duplicate, name: Registry.Mora}
    ]

    opts = [strategy: :one_for_one, name: Mora.Supervisor]
    return_value = Supervisor.start_link(children, opts)
    Logger.info("Started Mora")
    return_value
  end
end
