defmodule Mora.Events.SocketHandler do
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

  def websocket_handle({:text, json}, state) do
    Logger.debug("Handling websocket event notification")

    event = Poison.decode!(json, as: %Mora.Event{})
    Mora.Events.Dispatchers.Websocket.dispatch(event)
    new_state = %{registry_key: state.registry_key, count: state.count + 1}
    Logger.info("Websocket event notification handled")
    {:reply, {:text, "#{new_state.count}"}, new_state}
  end

  def websocket_info(info, state) do
    # not in use probably
    {:reply, {:text, Poison.encode!(info)}, state}
  end
end
