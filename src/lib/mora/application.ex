defmodule Mora.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application
  require Logger
  @impl true
  def start(_type, _args) do
    port = Application.fetch_env!(:mora, :http_port)
    Logger.info("Starting Mora DB on port #{port}")

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
      Mora.Events.TemporalQueue.Priority,
      Mora.Events.Database.Mnesia,
      Mora.Events.Dispatchers.Websocket,
      {Registry, keys: :duplicate, name: Registry.Mora}
    ]

    opts = [strategy: :one_for_one, name: Mora.Supervisor]
    return_value = Supervisor.start_link(children, opts)
    Logger.info("Started Mora")
    return_value
  end
end
