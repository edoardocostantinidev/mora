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
      {Plug.Cowboy,
       scheme: :http,
       plug: Moradb,
       options: [
         dispatch: PlugSocket.plug_cowboy_dispatch(Moradb.Api)
       ]},
      {Registry, keys: :duplicate, name: Registry.Moradb}
    ]

    opts = [strategy: :one_for_one, name: Moradb.Supervisor]
    return_value = Supervisor.start_link(children, opts)
    Logger.info("Started Mora DB ✅")
    return_value
  end
end
