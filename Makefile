.PHONY: changelog changelog-preview

changelog:
	git-cliff --output CHANGELOG.md

changelog-preview:
	git-cliff