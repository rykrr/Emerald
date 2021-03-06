#pragma once

#define loop for (;;)

#define for_range(var, hi) \
	for (int var = 0; var < hi; var++)

#define repeat(times)					\
	for (int __REPEAT__##__LINE__ = 0;	\
		__REPEAT__##__LINE__ < times;	\
		__REPEAT__##__LINE__++)
