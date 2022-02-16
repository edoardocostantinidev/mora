defmodule Mora.Test.Api.Routers.Events do
  @moduledoc """
  This module contains the event router tests.
  """
  use ExUnit.Case, async: true
  use Plug.Test
  alias Mora.Support.Generator
  alias Mora.Api
  import Mox
  @options Api.init([])
  setup :verify_on_exit!

  describe "Events Api Router" do
    test "post should process events through events service" do
      expect(Mora.Service.EventsMock, :process_events, fn _ -> :ok end)

      event = Generator.get_random_event()

      body =
        event
        |> Map.from_struct()
        |> Map.delete(:__meta__)
        |> Map.delete(:id)
        |> List.wrap()
        |> Poison.encode!()

      :post
      |> conn("/events", body)
      |> Api.call(@options)
    end
  end
end
