defmodule Mora.Api do
  @moduledoc """
  This module provides the API for the Mora server.
  """
  use Plug.Router
  use PlugSocket
  socket("/ws/events/[...]", Mora.Api.SocketHandler.Event)
  plug(:match)
  plug(:dispatch)
  forward("/events", to: Mora.Api.Routers.Event)
  forward("/status", to: Mora.Api.Routers.Status)
end