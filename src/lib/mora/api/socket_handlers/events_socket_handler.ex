defmodule Mora.Api.SocketHandler.Event do
  @behaviour :cowboy_websocket
  require Logger

  def init(req, _state) do
    [event_category] =
      req.path
      |> String.split(~r/\//)
      |> Enum.take(-1)

    Logger.info("Initializing websocket #{event_category}")
    state = %{registry_key: event_category, count: 0}
    Logger.info("Initialized websocket #{event_category}")
    {:cowboy_websocket, req, state}
  end

  def websocket_init(state) do
    Logger.info("Registering websocket connection #{state.registry_key}")

    Registry.Mora
    |> Registry.register(state.registry_key, {})

    Logger.info("Registered websocket connection #{state.registry_key}")
    {:ok, state}
  end

  def websocket_handle({:text, _json}, state) do
    Logger.error("websocket incoming handle not implemented")
    {:ok, state}
  end

  def websocket_info(info, state) do
    info = Map.put_new(info, :count, state.count + 1)
    info = Map.put_new(info, :received_from, node())
    state = Map.put(state, :count, state.count + 1)
    {:reply, {:text, Poison.encode!(info)}, state}
  end
end
