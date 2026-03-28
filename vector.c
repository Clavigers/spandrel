#include "vector.h"


void append(vector_t* self, int value)
{
	// Allocate more size for the vector
	self->data = realloc(self->data, (self->len + 1)*sizeof(int) );
	// Insert the new value 
	self->data[self->len] = value;
	// update the length of the vector
	self->len += 1;
}

int get(vector_t* self, size_t index)
{
	// First check if the desired index is out of bounds of the vector
	int result = -999;
	if (index >= self->len)
		return result;
	result = *(self->data + index);
	return result;
}

void set(vector_t* self, size_t index, int value)
{
	// First check if the desired index is out of bounds of the vector
	if (index >= self->len)
		return;
	self->data[index] = value;
}

/*
 * All the user needs to pass an initial array from stack memory/size of array
 * And the constructor will handle everything else
 */
vector_t* constructor(int* start_array, size_t length)
{
	// Allocate on the heap a new vector struct
	vector_t* new = (vector_t*)malloc(1 * sizeof(vector_t));
	// Point the self attribute to address of newly allocated vector
	new->self = new;
	new->len = length;

	// Initialize the data in the vector
	new->data = (int*)malloc(length * sizeof(int));
	for(int i=0; i<length; i++)
		new->data[i] = start_array[i];

	// Initialize the function pointers (methods)
	new->pfn_set = &set;
	new->pfn_get = &get;
	new->pfn_append = &append;

	return new;
}

// Destructor to dispose of the used memory
void destructor(vector_t* self)
{
	// This order is important
	free(self->data);
	free(self);
}
