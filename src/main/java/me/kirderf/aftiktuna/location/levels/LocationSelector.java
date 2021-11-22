package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.location.Location;

import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

public class LocationSelector {
	private final List<LocationCategory> remainingCategories = Locations.categories.stream()
			.map(LocationCategory::mutableCopy).collect(Collectors.toCollection(ArrayList::new));
	
	public Location getRandomLocation() {
		int i = GameInstance.RANDOM.nextInt(remainingCategories.size());
		LocationCategory category = remainingCategories.get(i);
		Location location = category.createAndRemoveRandom(GameInstance.RANDOM);
		if (category.isEmpty())
			remainingCategories.remove(category);
		return location;
	}
}