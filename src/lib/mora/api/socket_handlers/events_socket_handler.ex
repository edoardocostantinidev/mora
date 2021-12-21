defmodule Mora.Api.SocketHandler.Event do
  @moduledoc """
  This module contains the event websocket handlers for returning events to clients.
  """

  @pg_name "api:socket_handler:event"
  @pg_system_name "system:api:handlers"
  @behaviour Mora.CommonBehaviour.PgItem
  @behaviour :cowboy_websocket
  require Logger

  def init(req, _state) do
    [event_category] =
      req.path
      |> String.split(~r/\//)
      |> Enum.take(-1)

    state = %{category: event_category}
    Logger.info("Initialized websocket: #{event_category} for #{inspect(req.pid)}")
    {:cowboy_websocket, req, state, %{idle_timeout: :infinity}}
  end

  def websocket_init(state = %{category: category}) do
    :pg.join(pg_name(category), self())
    {:ok, state}
  end

  def websocket_handle({:text, _json}, state) do
    Logger.error("websocket incoming handle not implemented")
    {:ok, state}
  end

  def websocket_info(info, state) do
    info = Map.put_new(info, :received_from, node())
    {:reply, {:text, Poison.encode!(info)}, state}
  end

  def pg_name(category), do: @pg_name <> ":" <> category
  def pg_system_name(), do: @pg_system_name
end
