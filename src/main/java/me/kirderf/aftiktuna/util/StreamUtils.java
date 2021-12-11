package me.kirderf.aftiktuna.util;

import me.kirderf.aftiktuna.GameInstance;

import java.util.Comparator;
import java.util.Optional;
import java.util.stream.Stream;

public final class StreamUtils {
	
	public static <T> Optional<T> findRandomMin(Stream<T> stream, Comparator<? super T> comparator) {
		return stream.map(t -> new Entry<>(t, GameInstance.RANDOM.nextFloat()))
				.min(Comparator.<Entry<T>, T>comparing(Entry::element, comparator).thenComparing(Entry::randValue, Float::compare))
				.map(Entry::element);
	}
	
	public static <T> Stream<T> sortedWithRandomTiebreaker(Stream<T> stream, Comparator<? super T> comparator) {
		return stream.map(t -> new Entry<>(t, GameInstance.RANDOM.nextFloat()))
				.sorted(Comparator.<Entry<T>, T>comparing(Entry::element, comparator).thenComparing(Entry::randValue, Float::compare))
				.map(Entry::element);
	}
	
	private record Entry<T>(T element, float randValue) {}
}