defmodule Mora.Test.Api do
  @moduledoc """
  This module is for testing the API.
  """

  use ExUnit.Case, async: true

  use Plug.Test
  alias Mora.Api
  alias Mora.Support.Generator

  @options Api.init([])

  describe "status api" do
    test "should_return_200_OK_if_get_on_status" do
      conn = :get |> conn("/status/health", %{}) |> Api.call(@options)
      assert(conn.status == 200)
      assert(conn.resp_body == "OK")
    end

    test "should_return_200_with_no_body_if_no_queue_active" do
      :pg.get_members("system:temporal_queues")
      |> Enum.each(fn pid -> :pg.leave("system:temporal_queues", pid) end)

      conn = :get |> conn("/status/queues", %{}) |> Api.call(@options)
      assert(conn.status == 200)
      assert(conn.resp_body == "[]")
    end

    test "should_return_200_queue_description_if_get_on_queues_with_one_active_queue" do
      event = Generator.get_random_event()

      body =
        event
        |> Map.from_struct()
        |> Map.delete(:__meta__)
        |> List.wrap()
        |> Poison.encode!()

      :post
      |> conn("/events", body)
      |> Api.call(@options)

      conn =
        :get
        |> conn("/status/queues", %{})
        |> Api.call(@options)

      assert(conn.status == 200)

      response = Poison.decode!(conn.resp_body)

      assert(Enum.count(response) == 1)
    end
  end
end
