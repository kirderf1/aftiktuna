package me.kirderf.aftiktuna.util;

import java.util.Optional;
import java.util.function.Function;
import java.util.function.Predicate;
import java.util.stream.Stream;

@SuppressWarnings("unused")
@FunctionalInterface
public interface OptionalFunction<T, R> extends Function<T, Optional<R>> {
	
	static <T> OptionalFunction<T, T> of(Predicate<T> predicate) {
		return t -> predicate.test(t) ? Optional.of(t) : Optional.empty();
	}
	
	static <T, R> OptionalFunction<T, R> cast(Class<R> clazz) {
		return t -> clazz.isInstance(t) ? Optional.of(clazz.cast(t)) : Optional.empty();
	}
	
	default <S> OptionalFunction<T, R> filter(Predicate<R> predicate) {
		return t -> this.apply(t).filter(predicate);
	}
	
	default <S> OptionalFunction<T, S> map(Function<R, S> other) {
		return t -> this.apply(t).map(other);
	}
	
	default <S> OptionalFunction<T, S> flatMap(OptionalFunction<R, S> other) {
		return t -> this.apply(t).flatMap(other);
	}
	
	default Function<T, Stream<R>> toStream() {
		return t -> this.apply(t).stream();
	}
	
	default Predicate<T> toPredicate() {
		return t -> this.apply(t).isPresent();
	}
}