defmodule Mora.Support.Generator do
  @moduledoc """
  This module provides a simple way to generate random values.
  """
  def get_random_event do
    get_random_event(1_682_235_256_700, 1_692_255_256_733)
  end

  def get_random_event(fire_at_min, fire_at_max, category \\ random_string(8)) do
    event =
      struct(Mora.Model.Event,
        fireAt: :rand.uniform(fire_at_max - fire_at_min) + fire_at_min,
        createdAt: :rand.uniform(fire_at_max - fire_at_min) + fire_at_min,
        category: category,
        data: %{
          foo: "bar"
        }
      )

    event_hash = :erlang.phash2(event)
    Map.put(event, :id, "#{event.createdAt}-#{event.fireAt}-#{event_hash}")
  end

  def random_string(length) do
    :crypto.strong_rand_bytes(length) |> Base.url_encode64() |> binary_part(0, length)
  end
end
