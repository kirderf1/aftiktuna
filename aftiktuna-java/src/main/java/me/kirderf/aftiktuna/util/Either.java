package me.kirderf.aftiktuna.util;

import java.util.Optional;
import java.util.function.Consumer;

@SuppressWarnings("unused")
public abstract class Either<A, B> {
	
	public static <A, B> Either<A, B> left(A element) {
		return new Left<>(element);
	}
	
	public static <A, B> Either<A, B> right(B element) {
		return new Right<>(element);
	}
	
	public abstract boolean isLeft();
	
	public abstract boolean isRight();
	
	public abstract Optional<A> getLeft();
	
	public abstract Optional<B> getRight();
	
	public abstract void run(Consumer<A> left, Consumer<B> right);
	
	private static class Left<A, B> extends Either<A, B> {
		private final A element;
		
		private Left(A element) {
			this.element = element;
		}
		
		@Override
		public boolean isLeft() {
			return true;
		}
		
		@Override
		public boolean isRight() {
			return false;
		}
		
		@Override
		public Optional<A> getLeft() {
			return Optional.of(element);
		}
		
		@Override
		public Optional<B> getRight() {
			return Optional.empty();
		}
		
		@Override
		public void run(Consumer<A> left, Consumer<B> right) {
			left.accept(element);
		}
	}
	
	private static class Right<A, B> extends Either<A, B> {
		private final B element;
		
		private Right(B element) {
			this.element = element;
		}
		
		@Override
		public boolean isLeft() {
			return false;
		}
		
		@Override
		public boolean isRight() {
			return true;
		}
		
		@Override
		public Optional<A> getLeft() {
			return Optional.empty();
		}
		
		@Override
		public Optional<B> getRight() {
			return Optional.of(element);
		}
		
		@Override
		public void run(Consumer<A> left, Consumer<B> right) {
			right.accept(element);
		}
	}
}