defmodule Moradb.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application
  require Logger
  @impl true
  def start(_type, _args) do
    Logger.info("Starting Mora DB 🚀")

    children = [
      {
        Plug.Cowboy,
        scheme: :http, plug: Moradb.Api, dispatch: dispatch(), options: [port: 4000]
      },
      {Registry, keys: :duplicate, name: Registry.Moradb}
    ]

    opts = [strategy: :one_for_one, name: Moradb.Supervisor]
    return_value = Supervisor.start_link(children, opts)
    Logger.info("Started Mora DB ✅")
    return_value
  end

  defp dispatch() do
    [
      {
        :_,
        [
          {"/ws/[...]", Moradb.SocketHandlers.Events, []},
          {:_, Plug.Cowboy.Handler, {Moradb.Routers.Events, []}}
        ]
      }
    ]
  end
end
